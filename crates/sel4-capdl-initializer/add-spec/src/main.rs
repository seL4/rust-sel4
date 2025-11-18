//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs;

use anyhow::Result;
use clap::{Arg, ArgAction, Command};

use sel4_capdl_initializer_add_spec::{ObjectNamesLevel, add_spec};

fn main() -> Result<()> {
    let args = Args::parse()?;

    if args.verbose {
        eprintln!("{args:#?}");
    }

    let initializer_without_spec_buf = fs::read(&args.initializer_elf_path)?;

    let input_spec_json = fs::read_to_string(&args.spec_json_path)?;
    let input_spec = serde_json::from_str(&input_spec_json)?;

    let rendered_initializer_elf_buf = add_spec(
        &initializer_without_spec_buf,
        &input_spec,
        &[&args.fill_dir_path],
        &args.object_names_level,
        args.embed_frames,
        args.deflate,
        args.initializer_verbosity,
    );

    fs::write(&args.out_file_path, rendered_initializer_elf_buf)?;
    Ok(())
}

#[derive(Debug)]
pub struct Args {
    pub initializer_elf_path: String,
    pub spec_json_path: String,
    pub fill_dir_path: String,
    pub out_file_path: String,
    pub object_names_level: ObjectNamesLevel,
    pub embed_frames: bool,
    pub deflate: bool,
    pub initializer_verbosity: u8,
    pub verbose: bool,
}

const DEFAULT_INITIALIZER_VERBOSITY: u8 = 3; // log::LevelFilter::Info

impl Args {
    pub fn parse() -> Result<Self> {
        let matches = Command::new("")
            .arg(
                Arg::new("initializer_elf")
                    .short('e')
                    .value_name("INITIALIZER")
                    .required(true),
            )
            .arg(
                Arg::new("spec_json")
                    .short('f')
                    .value_name("SPEC_FILE")
                    .required(true),
            )
            .arg(
                Arg::new("fill_dir")
                    .short('d')
                    .value_name("FILL_DIR")
                    .required(true),
            )
            .arg(
                Arg::new("out_file")
                    .short('o')
                    .value_name("OUT_FILE")
                    .required(true),
            )
            .arg(
                Arg::new("object_names_level")
                    .long("object-names-level")
                    .short('n')
                    .value_name("OBJECT_NAMES_LEVEL")
                    .value_parser(clap::value_parser!(u32).range(..=2)),
            )
            .arg(
                Arg::new("no_embed_frames")
                    .long("no-embed-frames")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("no_deflate")
                    .long("no-deflate")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("initializer_verbosity")
                    .long("initializer-verbosity")
                    .value_name("VERBOSITY")
                    .value_parser(clap::value_parser!(u8).range(..=5)),
            )
            .arg(Arg::new("verbose").short('v').action(ArgAction::SetTrue))
            .get_matches();

        let initializer_elf_path = matches
            .get_one::<String>("initializer_elf")
            .unwrap()
            .to_owned();
        let spec_json_path = matches.get_one::<String>("spec_json").unwrap().to_owned();
        let fill_dir_path = matches.get_one::<String>("fill_dir").unwrap().to_owned();
        let out_file_path = matches.get_one::<String>("out_file").unwrap().to_owned();

        let object_names_level = matches
            .get_one::<u32>("object_names_level")
            .map(|val| match val {
                0 => ObjectNamesLevel::None,
                1 => ObjectNamesLevel::JustTcbs,
                2 => ObjectNamesLevel::All,
                _ => panic!(),
            })
            .unwrap_or(ObjectNamesLevel::JustTcbs);

        let embed_frames = !*matches.get_one::<bool>("no_embed_frames").unwrap();
        let deflate = !*matches.get_one::<bool>("no_deflate").unwrap();
        let initializer_verbosity = *matches
            .get_one::<u8>("initializer_verbosity")
            .unwrap_or(&DEFAULT_INITIALIZER_VERBOSITY);

        let verbose = *matches.get_one::<bool>("verbose").unwrap();

        Ok(Self {
            initializer_elf_path,
            spec_json_path,
            fill_dir_path,
            out_file_path,
            object_names_level,
            embed_frames,
            deflate,
            initializer_verbosity,
            verbose,
        })
    }
}
