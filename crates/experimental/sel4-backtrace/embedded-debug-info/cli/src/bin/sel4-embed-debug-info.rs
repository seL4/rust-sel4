//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs;
use std::io;

use clap::Parser;
use object::read::elf::ElfFile;

use sel4_patch_elf::{FileHeaderExt, Patching};
use sel4_phdrs_constants::PT_SEL4_EMBEDDED_DEBUG_INFO;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long, short = 'i')]
    image_elf: String,
    #[arg(long, short = 'd')]
    debug_info_elf: String,
    #[arg(long, short = 'o')]
    out_elf: String,
}

fn main() -> Result<(), io::Error> {
    let cli = Cli::parse();

    let image_elf_buf = fs::read(&cli.image_elf)?;
    let debug_info_elf_buf = fs::read(&cli.debug_info_elf)?;

    let out_elf_buf = match object::File::parse(&*image_elf_buf).unwrap() {
        object::File::Elf32(image_elf) => with_bit_width(&image_elf, &debug_info_elf_buf),
        object::File::Elf64(image_elf) => with_bit_width(&image_elf, &debug_info_elf_buf),
        _ => {
            panic!()
        }
    };

    fs::write(&cli.out_elf, out_elf_buf)
}

fn with_bit_width<T: FileHeaderExt>(image_elf: &ElfFile<T>, content: &[u8]) -> Vec<u8> {
    let mut patching = Patching::new(image_elf);
    patching.add_data_segment_with_meta_phdr(PT_SEL4_EMBEDDED_DEBUG_INFO, 1, content);
    patching.finalize()
}
