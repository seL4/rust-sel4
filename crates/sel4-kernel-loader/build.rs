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
use sel4_config::{sel4_cfg_if, sel4_cfg_str, sel4_cfg_usize};
use sel4_kernel_loader_embed_page_tables::{
    schemes, LeafLocation, Region, RegionsBuilder, Scheme, SchemeHelpers,
};
use sel4_platform_info::PLATFORM_INFO;
use sel4_rustfmt_helper::Rustfmt;

pub const SEL4_KERNEL_ENV: &str = "SEL4_KERNEL";

sel4_cfg_if! {
    if #[cfg(WORD_SIZE = "64")] {
        type FileHeader = object::elf::FileHeader64<Endianness>;
    } else if #[cfg(WORD_SIZE = "32")] {
        type FileHeader = object::elf::FileHeader32<Endianness>;
    }
}

sel4_cfg_if! {
    if #[cfg(SEL4_ARCH = "aarch64")] {
        type SchemeImpl = schemes::AArch64;
    } else if #[cfg(SEL4_ARCH = "riscv64")] {
        sel4_cfg_if! {
            if #[cfg(PT_LEVELS = "3")] {
                type SchemeImpl = schemes::Riscv64Sv39;
            }
        }
    } else if #[cfg(SEL4_ARCH = "riscv32")] {
        sel4_cfg_if! {
            if #[cfg(PT_LEVELS = "2")] {
                type SchemeImpl = schemes::Riscv32Sv32;
            }
        }
    }
}

const GRANULE_SIZE: u64 = 1 << SchemeImpl::PAGE_BITS;

const KERNEL_HEADROOM: u64 = 256 * 1024; // TODO: make configurable

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    {
        let asm_files = []
            .into_iter()
            .chain(glob::glob(&format!("asm/{}/*.S", sel4_cfg_str!(ARCH))).unwrap())
            .chain(glob::glob(&format!("asm/{}/*.S", sel4_cfg_str!(SEL4_ARCH))).unwrap())
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

    if let "aarch64" | "aarch32" = sel4_cfg_str!(SEL4_ARCH) {
        let out_path = PathBuf::from(&out_dir).join("loader_translation_tables.rs");
        fs::write(&out_path, mk_loader_map()).unwrap();
        Rustfmt::detect().format(&out_path);
    }

    let kernel_path = get_with_sel4_prefix_relative_fallback(SEL4_KERNEL_ENV, "bin/kernel.elf");
    let kernel_bytes = fs::read(kernel_path).unwrap();
    let kernel_elf = ElfFile::<FileHeader, _>::parse(kernel_bytes.as_slice()).unwrap();
    let kernel_phys_addr_range = elf_phys_addr_range(&kernel_elf);
    let kernel_phys_to_virt_offset = elf_phys_to_vaddr_offset(&kernel_elf);

    let loader_phys_start =
        (kernel_phys_addr_range.end + KERNEL_HEADROOM).next_multiple_of(GRANULE_SIZE);

    {
        let out_path = PathBuf::from(&out_dir).join("kernel_translation_tables.rs");
        fs::write(
            &out_path,
            mk_kernel_map(kernel_phys_addr_range, kernel_phys_to_virt_offset),
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

fn mk_loader_map() -> String {
    let mut regions = RegionsBuilder::<SchemeImpl>::new();
    for range in PLATFORM_INFO.memory.iter() {
        let range = range.start.into()..range.end.into();
        regions = regions.insert(Region::valid(
            range,
            SchemeImpl::mk_normal_leaf_for_loader_map,
        ));
    }
    for range in get_device_regions() {
        regions = regions.insert(Region::valid(
            range,
            SchemeImpl::mk_device_leaf_for_loader_map,
        ));
    }

    let toks = regions.build().construct_table().embed(
        format_ident!("loader_level_0_table"),
        format_ident!("sel4_kernel_loader_embed_page_tables_runtime"),
    );

    format!("{toks}")
}

// HACK
fn get_device_regions() -> Vec<Range<u64>> {
    let page = |start| start..start + GRANULE_SIZE;
    match sel4_cfg_str!(PLAT) {
        "qemu-arm-virt" => vec![page(0x0900_0000)],
        "bcm2711" => vec![page(0x0000_0000), page(0xfe21_5000)],
        "spike" => vec![],
        "qemu-riscv-virt" => vec![],
        _ => panic!("unsupported platform"),
    }
}

fn mk_kernel_map(kernel_phys_addr_range: Range<u64>, kernel_phys_to_virt_offset: u64) -> String {
    let virt_start = kernel_phys_addr_range
        .start
        .wrapping_add(kernel_phys_to_virt_offset);
    let virt_end = kernel_phys_addr_range
        .end
        .wrapping_add(kernel_phys_to_virt_offset);
    let virt_map_end =
        virt_end.next_multiple_of(1 << SchemeHelpers::<SchemeImpl>::largest_leaf_size_bits());

    let regions = RegionsBuilder::<SchemeImpl>::new()
        .insert(Region::valid(
            0..virt_start,
            SchemeImpl::mk_identity_leaf_for_kernel_map,
        ))
        .insert(Region::valid(virt_start..virt_map_end, move |loc| {
            SchemeImpl::mk_kernel_leaf_for_kernel_map(kernel_phys_to_virt_offset, loc)
        }));

    let toks = regions.build().construct_table().embed(
        format_ident!("kernel_boot_level_0_table"),
        format_ident!("sel4_kernel_loader_embed_page_tables_runtime"),
    );

    format!("{}", toks)
}

trait SchemeExt: Scheme {
    fn mk_normal_leaf_for_loader_map(_loc: LeafLocation) -> Self::LeafDescriptor {
        unimplemented!()
    }

    fn mk_device_leaf_for_loader_map(_loc: LeafLocation) -> Self::LeafDescriptor {
        unimplemented!()
    }

    fn mk_identity_leaf_for_kernel_map(loc: LeafLocation) -> Self::LeafDescriptor;

    fn mk_kernel_leaf_for_kernel_map(
        phys_to_virt_offset: u64,
        loc: LeafLocation,
    ) -> Self::LeafDescriptor;
}

impl SchemeExt for schemes::AArch64 {
    fn mk_normal_leaf_for_loader_map(loc: LeafLocation) -> Self::LeafDescriptor {
        loc.map_identity::<schemes::AArch64>()
            .set_access_flag(true)
            .set_attribute_index(4) // select MT_NORMAL
            .set_shareability(AARCH64_NORMAL_SHAREABILITY)
    }

    fn mk_device_leaf_for_loader_map(loc: LeafLocation) -> Self::LeafDescriptor {
        loc.map_identity::<schemes::AArch64>()
            .set_access_flag(true)
            .set_attribute_index(0) // select MT_DEVICE_nGnRnE
    }

    fn mk_identity_leaf_for_kernel_map(loc: LeafLocation) -> Self::LeafDescriptor {
        loc.map_identity::<schemes::AArch64>()
            .set_access_flag(true)
            .set_attribute_index(0) // select MT_DEVICE_nGnRnE
    }

    fn mk_kernel_leaf_for_kernel_map(
        phys_to_virt_offset: u64,
        loc: LeafLocation,
    ) -> Self::LeafDescriptor {
        loc.map::<schemes::AArch64>(|vaddr| virt_to_phys(vaddr, phys_to_virt_offset))
            .set_access_flag(true)
            .set_attribute_index(4) // select MT_NORMAL
            .set_shareability(AARCH64_NORMAL_SHAREABILITY)
    }
}

const AARCH64_NORMAL_SHAREABILITY: u64 = if sel4_cfg_usize!(MAX_NUM_NODES) > 1 {
    0b11
} else {
    0b00
};

impl SchemeExt for schemes::Riscv64Sv39 {
    fn mk_identity_leaf_for_kernel_map(loc: LeafLocation) -> Self::LeafDescriptor {
        loc.map_identity::<Self>()
    }

    fn mk_kernel_leaf_for_kernel_map(
        phys_to_virt_offset: u64,
        loc: LeafLocation,
    ) -> Self::LeafDescriptor {
        loc.map::<Self>(|vaddr| virt_to_phys(vaddr, phys_to_virt_offset))
    }
}

impl SchemeExt for schemes::Riscv32Sv32 {
    fn mk_identity_leaf_for_kernel_map(loc: LeafLocation) -> Self::LeafDescriptor {
        loc.map_identity::<Self>()
    }

    fn mk_kernel_leaf_for_kernel_map(
        phys_to_virt_offset: u64,
        loc: LeafLocation,
    ) -> Self::LeafDescriptor {
        loc.map::<Self>(|vaddr| virt_to_phys(vaddr, phys_to_virt_offset))
    }
}

// // //

fn elf_phys_addr_range<'a, R: ReadRef<'a>>(elf: &ElfFile<'a, FileHeader, R>) -> Range<u64> {
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
        .map(|phdr| {
            phdr.p_paddr(endian)
                .checked_add(phdr.p_memsz(endian))
                .unwrap()
        })
        .max()
        .unwrap();
    virt_min.into()..virt_max.into()
}

fn elf_phys_to_vaddr_offset<'a, R: ReadRef<'a>>(elf: &ElfFile<'a, FileHeader, R>) -> u64 {
    let endian = elf.endian();
    unified(
        elf.raw_segments()
            .iter()
            .filter(|phdr| phdr.p_type(endian) == PT_LOAD)
            .map(|phdr| {
                let paddr = phdr.p_paddr(endian).into();
                let vaddr_with_extension: u64 = phdr.p_vaddr(endian).into();
                let vaddr = vaddr_with_extension & SchemeHelpers::<SchemeImpl>::vaddr_mask();
                phys_to_virt_offset_for(paddr, vaddr)
            }),
    )
}

fn phys_to_virt_offset_for(paddr: u64, vaddr: u64) -> u64 {
    vaddr.wrapping_sub(paddr)
}

fn virt_to_phys(vaddr: u64, phys_to_virt_offset: u64) -> u64 {
    vaddr.wrapping_sub(phys_to_virt_offset)
}

fn unified<T: Eq>(mut it: impl Iterator<Item = T>) -> T {
    let first = it.next().unwrap();
    assert!(it.all(|subsequent| subsequent == first));
    first
}
