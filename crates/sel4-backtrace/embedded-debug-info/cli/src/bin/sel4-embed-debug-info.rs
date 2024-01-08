//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs;
use std::io;

use clap::{Arg, Command};
use num::{NumCast, One, PrimInt, Zero};

use sel4_render_elf_with_data::{
    ConcreteFileHeader32, ConcreteFileHeader64, ElfBitWidth, FileHeaderExt, Input,
    SymbolicInjection, SymbolicValue,
};

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

    let image_elf = fs::read(image_elf_path)?;
    let debug_info_elf = fs::read(debug_info_elf_path)?;

    let out_elf = match ElfBitWidth::detect(&image_elf).unwrap() {
        ElfBitWidth::Elf32 => with_bit_width::<ConcreteFileHeader32>(&image_elf, &debug_info_elf),
        ElfBitWidth::Elf64 => with_bit_width::<ConcreteFileHeader64>(&image_elf, &debug_info_elf),
    };

    fs::write(out_elf_path, out_elf)
}

fn with_bit_width<T>(image_elf: &[u8], content: &[u8]) -> Vec<u8>
where
    T: FileHeaderExt,
    T::Word: PrimInt,
    T::Sword: PrimInt,
{
    let content_len = NumCast::from(content.len()).unwrap();
    let mut input = Input::<T>::default();
    input.symbolic_injections.push(SymbolicInjection {
        align_modulus: T::Word::one(),
        align_residue: T::Word::zero(),
        content,
        memsz: content_len,
        patches: vec![(
            "embedded_debug_info_start".to_owned(),
            SymbolicValue {
                addend: T::Sword::zero(),
            },
        )],
    });
    input
        .concrete_patches
        .push(("embedded_debug_info_size".to_owned(), content_len));
    input.render_with_data(image_elf).unwrap()
}
