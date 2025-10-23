//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs;
use std::io;

use clap::{Arg, Command};
use num::NumCast;

use sel4_synthetic_elf::{Builder, PatchValue, Segment, object};

fn main() -> Result<(), io::Error> {
    let matches = Command::new("")
        .arg(
            Arg::new("image_elf")
                .short('i')
                .value_name("IMAGE_ELF")
                .required(true),
        )
        .arg(
            Arg::new("debug_info_elf")
                .short('d')
                .value_name("DEBUG_INFO_ELF")
                .required(true),
        )
        .arg(
            Arg::new("out_elf")
                .short('o')
                .value_name("OUT_ELF")
                .required(true),
        )
        .arg(
            Arg::new("object_names_level")
                .long("object-names-level")
                .short('n')
                .value_name("OBJECT_NAMES_LEVEL"),
        )
        .get_matches();

    let image_elf_path = matches.get_one::<String>("image_elf").unwrap().to_owned();
    let debug_info_elf_path = matches
        .get_one::<String>("debug_info_elf")
        .unwrap()
        .to_owned();
    let out_elf_path = matches.get_one::<String>("out_elf").unwrap().to_owned();

    let image_elf_buf = fs::read(image_elf_path)?;
    let debug_info_elf_buf = fs::read(debug_info_elf_path)?;

    let out_elf_buf = match object::File::parse(&*image_elf_buf).unwrap() {
        object::File::Elf32(image_elf) => with_bit_width(&image_elf, &debug_info_elf_buf),
        object::File::Elf64(image_elf) => with_bit_width(&image_elf, &debug_info_elf_buf),
        _ => {
            panic!()
        }
    };

    fs::write(out_elf_path, out_elf_buf)
}

fn with_bit_width<T: object::read::elf::FileHeader<Word: NumCast + PatchValue>>(
    image_elf: &object::read::elf::ElfFile<T>,
    content: &[u8],
) -> Vec<u8> {
    let mut builder = Builder::new(image_elf).unwrap();

    builder.discard_p_align(true);

    let vaddr = builder.footprint().unwrap().end.next_multiple_of(4096);

    builder.add_segment(Segment::simple(vaddr, content.into()));

    builder
        .patch_word_with_cast("embedded_debug_info_start", vaddr)
        .unwrap();
    builder
        .patch_word_with_cast("embedded_debug_info_size", content.len())
        .unwrap();

    builder.build().unwrap()
}
