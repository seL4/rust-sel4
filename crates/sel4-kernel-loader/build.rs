#![feature(int_roundings)]

use std::env;
use std::fs;
use std::ops::Range;
use std::path::PathBuf;

use object::{
    elf::PT_LOAD,
    endian::Endianness,
    read::elf::{ElfFile, ProgramHeader},
    ReadRef,
};
use quote::format_ident;

use sel4_build_env::{get_libsel4_include_dirs, get_with_sel4_prefix_relative_fallback};
use sel4_kernel_loader_embed_page_tables::{AArch64, LeafLocation, Region, RegionsBuilder};
use sel4_platform_info::PLATFORM_INFO;
use sel4_rustfmt_helper::Rustfmt;

sel4_config::sel4_cfg_if! {
    if #[cfg(WORD_SIZE = "64")] {
        type FileHeader<T> = object::elf::FileHeader64<T>;
    } else if #[cfg(WORD_SIZE = "32")] {
        type FileHeader<T> = object::elf::FileHeader32<T>;
    }
}

pub const SEL4_KERNEL_ENV: &str = "SEL4_KERNEL";

const KERNEL_HEADROOM: u64 = 256 * 1024; // TODO
const GRANULE_SIZE: u64 = 4096;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    {
        let asm_files = glob::glob("asm/aarch64/*.S")
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        cc::Build::new()
            .files(&asm_files)
            .includes(get_libsel4_include_dirs())
            .compile("asm");

        for path in &asm_files {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    {
        let out_path = PathBuf::from(&out_dir).join("loader_translation_tables.rs");
        fs::write(&out_path, mk_loader_translation_tables()).unwrap();
        Rustfmt::detect().format(&out_path);
    }

    let kernel_path = get_with_sel4_prefix_relative_fallback(SEL4_KERNEL_ENV, "bin/kernel.elf");
    let kernel_bytes = fs::read(kernel_path).unwrap();
    let kernel_elf = ElfFile::<FileHeader<Endianness>, _>::parse(kernel_bytes.as_slice()).unwrap();
    let kernel_phys_addr_range = elf_phys_addr_range(&kernel_elf);
    let kernel_phys_to_virt_offset = elf_phys_to_vaddr_offset(&kernel_elf);

    let loader_phys_start =
        (kernel_phys_addr_range.end + KERNEL_HEADROOM).next_multiple_of(GRANULE_SIZE);

    {
        let out_path = PathBuf::from(&out_dir).join("kernel_translation_tables.rs");
        fs::write(
            &out_path,
            mk_kernel_translation_tables(kernel_phys_addr_range.start, kernel_phys_to_virt_offset),
        )
        .unwrap();
        Rustfmt::detect().format(&out_path);
    }

    // Note that -Ttext={} is incompatible with --no-rosegment (no error),
    // just bad output. See the "Default program headers" section of:
    // https://maskray.me/blog/2020-12-19-lld-and-gnu-linker-incompatibilities
    println!(
        "cargo:rustc-link-arg=--image-base=0x{:x}",
        loader_phys_start
    );

    println!("cargo:rustc-link-arg=-z");
    println!("cargo:rustc-link-arg=max-page-size=4096");

    // No use in loader.
    // Remove unnecessary alignment gap between segments.
    println!("cargo:rustc-link-arg=--no-rosegment");
}

// // //

fn mk_loader_translation_tables() -> String {
    let normal_shareability = if sel4_config::sel4_cfg_usize!(MAX_NUM_NODES) > 1 {
        0b11
    } else {
        0b00
    };

    let mk_normal_entry = move |params: LeafLocation| {
        params
            .map_identity::<AArch64>()
            .set_access_flag(true)
            .set_attribute_index(4) // select MT_NORMAL
            .set_shareability(normal_shareability)
    };

    let mk_device_entry = |params: LeafLocation| {
        params
            .map_identity::<AArch64>()
            .set_access_flag(true)
            .set_attribute_index(0) // select MT_DEVICE_nGnRnE
    };

    let mut regions = RegionsBuilder::<AArch64>::new();
    for range in PLATFORM_INFO.memory.iter() {
        let range = range.start..range.end;
        regions = regions.insert(Region::valid(range, mk_normal_entry));
    }
    for range in get_device_regions() {
        regions = regions.insert(Region::valid(range, mk_device_entry));
    }

    let toks = regions
        .build()
        .construct_table()
        .embed(format_ident!("loader_level_0_table"));
    format!("{toks}")
}

// HACK
fn get_device_regions() -> Vec<Range<u64>> {
    let page = |start| start..start + 4096;
    sel4_config::sel4_cfg_if! {
        if #[cfg(PLAT_QEMU_ARM_VIRT)] {
            vec![
                page(0x0900_0000),
            ]
        } else if #[cfg(PLAT_BCM2711)] {
            vec![
                page(0x0000_0000),
                page(0xfe21_5000),
            ]
        } else {
            compile_error!("Unsupported platform");
        }
    }
}

// // //

fn mk_kernel_translation_tables(kernel_phys_start: u64, kernel_phys_to_virt_offset: i64) -> String {
    let phys_to_virt_offset = kernel_phys_to_virt_offset;
    let virt_start =
        u64::try_from(i64::try_from(kernel_phys_start).unwrap() + phys_to_virt_offset).unwrap();
    let virt_end = virt_start.next_multiple_of(1 << 39);

    let normal_shareability = if sel4_config::sel4_cfg_usize!(MAX_NUM_NODES) > 1 {
        3
    } else {
        0
    };

    let identity_map = |params: LeafLocation| {
        params
            .map_identity::<AArch64>()
            .set_access_flag(true)
            .set_attribute_index(0) // select MT_DEVICE_nGnRnE
    };

    let kernel_map = move |params: LeafLocation| {
        params
            .map::<AArch64>(|vaddr| virt_to_phys(vaddr, phys_to_virt_offset))
            .set_access_flag(true)
            .set_attribute_index(4) // select MT_NORMAL
            .set_shareability(normal_shareability)
    };

    let regions = RegionsBuilder::<AArch64>::new()
        .insert(Region::valid(0..virt_start, identity_map))
        .insert(Region::valid(virt_start..virt_end, kernel_map));

    let toks = regions
        .build()
        .construct_table()
        .embed(format_ident!("kernel_boot_level_0_table"));
    format!("{}", toks)
}

//

fn elf_phys_addr_range<'a, T: ReadRef<'a>>(
    elf: &ElfFile<'a, FileHeader<Endianness>, T>,
) -> Range<u64> {
    let endian = elf.endian();
    let virt_min = elf
        .raw_segments()
        .iter()
        .filter(|phdr| phdr.p_type(endian) == PT_LOAD)
        .map(|phdr| phdr.p_paddr(endian))
        .min()
        .unwrap();
    let virt_max = elf
        .raw_segments()
        .iter()
        .filter(|phdr| phdr.p_type(endian) == PT_LOAD)
        .map(|phdr| phdr.p_paddr(endian) + phdr.p_memsz(endian))
        .max()
        .unwrap();
    virt_min..virt_max
}

fn elf_phys_to_vaddr_offset<'a, T: ReadRef<'a>>(
    elf: &ElfFile<'a, FileHeader<Endianness>, T>,
) -> i64 {
    let endian = elf.endian();
    unified(
        elf.raw_segments()
            .iter()
            .filter(|phdr| phdr.p_type(endian) == PT_LOAD)
            .map(|phdr| phys_to_virt_offset_for(phdr.p_paddr(endian), phdr.p_vaddr(endian))),
    )
}

fn phys_to_virt_offset_for(paddr: u64, vaddr: u64) -> i64 {
    i64::try_from(vaddr).unwrap() - i64::try_from(paddr).unwrap()
}

fn virt_to_phys(vaddr: u64, phys_to_virt_offset: i64) -> u64 {
    u64::try_from(i64::try_from(vaddr).unwrap() - phys_to_virt_offset).unwrap()
}

fn unified<T: Eq>(mut it: impl Iterator<Item = T>) -> T {
    let first = it.next().unwrap();
    assert!(it.all(|subsequent| subsequent == first));
    first
}
