//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(clippy::useless_conversion)]

use std::fs;
use std::ops::Range;

use object::{
    ReadRef,
    elf::PT_LOAD,
    endian::Endianness,
    read::elf::{ElfFile, FileHeader, ProgramHeader},
};

use sel4_build_env::{get_libsel4_include_dirs, get_with_sel4_prefix_relative_fallback};
use sel4_config::{sel4_cfg, sel4_cfg_str};

pub const SEL4_KERNEL_ENV: &str = "SEL4_KERNEL";

#[sel4_cfg(WORD_SIZE = "64")]
type FileHeaderImpl = object::elf::FileHeader64<Endianness>;

#[sel4_cfg(WORD_SIZE = "32")]
type FileHeaderImpl = object::elf::FileHeader32<Endianness>;

const KERNEL_HEADROOM: u64 = 256 * 1024; // TODO: make configurable

const GRANULE_SIZE: u64 = 4096;

fn main() {
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
            println!("cargo::rerun-if-changed={}", path.display());
        }
    }

    let kernel_path = get_with_sel4_prefix_relative_fallback(SEL4_KERNEL_ENV, "bin/kernel.elf");
    let kernel_bytes = fs::read(kernel_path).unwrap();
    let kernel_elf = ElfFile::<FileHeaderImpl, _>::parse(kernel_bytes.as_slice()).unwrap();
    let kernel_phys_addr_range = phys_addr_range(&kernel_elf);

    let loader_phys_start =
        (kernel_phys_addr_range.end + KERNEL_HEADROOM).next_multiple_of(GRANULE_SIZE);

    // Note that -Ttext={} is incompatible with --no-rosegment (no error),
    // just bad output. See the "Default program headers" section of:
    // https://maskray.me/blog/2020-12-19-lld-and-gnu-linker-incompatibilities
    println!("cargo::rustc-link-arg=--image-base=0x{loader_phys_start:x}");

    println!("cargo::rustc-link-arg=-z");
    println!("cargo::rustc-link-arg=max-page-size=4096");

    // No use in loader.
    // Remove unnecessary alignment gap between segments.
    println!("cargo::rustc-link-arg=--no-rosegment");
}

// // //

fn phys_addr_range<'a, R: ReadRef<'a>>(elf: &ElfFile<'a, FileHeaderImpl, R>) -> Range<u64> {
    let endian = elf.endian();
    let virt_min = loadable_segments(elf)
        .map(|phdr| phdr.p_paddr(endian))
        .min()
        .unwrap();
    let virt_max = loadable_segments(elf)
        .map(|phdr| phdr.p_paddr(endian).strict_add(phdr.p_memsz(endian)))
        .max()
        .unwrap();
    virt_min.into()..virt_max.into()
}

fn loadable_segments<'data, 'file, R: ReadRef<'data>>(
    elf: &'file ElfFile<'data, FileHeaderImpl, R>,
) -> impl Iterator<Item = &'data <FileHeaderImpl as FileHeader>::ProgramHeader> {
    elf.elf_program_headers()
        .iter()
        .filter(|seg| seg.p_type(elf.endian()) == PT_LOAD)
}
