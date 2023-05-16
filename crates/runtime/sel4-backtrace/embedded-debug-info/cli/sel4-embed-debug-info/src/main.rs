use std::fs;
use std::io;

use clap::{App, Arg};

use sel4_render_elf_with_data::{Input, SymbolicInjection, SymbolicValue};

fn main() -> Result<(), io::Error> {
    let matches = App::new("")
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

    let content = &debug_info_elf;

    let out_elf = {
        let mut input = Input::default();
        input.symbolic_injections.push(SymbolicInjection {
            align_modulus: 1,
            align_residue: 0,
            content,
            memsz: content.len(),
            patches: vec![(
                "embedded_debug_info_start".to_owned(),
                SymbolicValue { addend: 0 },
            )],
        });
        input.concrete_patches.push((
            "embedded_debug_info_size".to_owned(),
            content.len().try_into().unwrap(),
        ));
        input.render_with_data(&image_elf).unwrap()
    };

    fs::write(out_elf_path, &out_elf)
}
