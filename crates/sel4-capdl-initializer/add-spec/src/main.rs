//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs;

use anyhow::Result;
use clap::Parser;

use sel4_capdl_initializer_add_spec::{ObjectNamesLevel, add_spec};

#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(long, short = 'e')]
    pub initializer_elf: String,
    #[arg(long, short = 'f')]
    pub spec_json: String,
    #[arg(long, short = 'd')]
    pub fill_dir: String,
    #[arg(long, short = 'o')]
    pub out_file: String,
    #[arg(long, short = 'n', value_parser = clap::value_parser!(u32).range(..=2))]
    pub object_names_level: u32,
    #[arg(long)]
    pub no_embed_frames: bool,
    #[arg(long)]
    pub no_deflate: bool,
    #[arg(long, value_parser = clap::value_parser!(u8).range(..=5), default_value_t = DEFAULT_INITIALIZER_VERBOSITY)]
    pub initializer_verbosity: u8,
    #[arg(long, short = 'v')]
    pub verbose: bool,
}

const DEFAULT_INITIALIZER_VERBOSITY: u8 = 3; // log::LevelFilter::Info

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        eprintln!("{cli:#?}");
    }

    let initializer_without_spec_buf = fs::read(&cli.initializer_elf)?;

    let input_spec_json = fs::read_to_string(&cli.spec_json)?;
    let input_spec = serde_json::from_str(&input_spec_json)?;

    let object_names_level = match cli.object_names_level {
        0 => ObjectNamesLevel::None,
        1 => ObjectNamesLevel::JustTcbs,
        2 => ObjectNamesLevel::All,
        _ => unreachable!(),
    };

    let rendered_initializer_elf_buf = add_spec(
        &initializer_without_spec_buf,
        &input_spec,
        &[&cli.fill_dir],
        &object_names_level,
        !cli.no_embed_frames,
        !cli.no_deflate,
        cli.initializer_verbosity,
    );

    fs::write(&cli.out_file, rendered_initializer_elf_buf)?;
    Ok(())
}
