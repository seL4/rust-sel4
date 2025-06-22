//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs::{self, File};

use anyhow::Result;
use num::{traits::WrappingSub, Integer, PrimInt};
use object::{
    elf::{FileHeader32, FileHeader64},
    read::elf::FileHeader,
    Endianness,
};
use serde::Serialize;

use sel4_config_types::Configuration;
use sel4_synthetic_elf::PatchValue;

mod args;
mod render_elf;
mod serialize_payload;

use args::Args;

fn main() -> Result<()> {
    let args = Args::parse()?;

    if args.verbose {
        eprintln!("{args:#?}");
    }

    let sel4_config: Configuration =
        serde_json::from_reader(File::open(&args.sel4_config_path).unwrap()).unwrap();

    let word_size = sel4_config
        .get("WORD_SIZE")
        .unwrap()
        .as_str()
        .unwrap()
        .parse::<usize>()
        .unwrap();

    match word_size {
        32 => continue_with_word_size::<FileHeader32<Endianness>>(&args),
        64 => continue_with_word_size::<FileHeader64<Endianness>>(&args),
        _ => {
            panic!()
        }
    }
}

fn continue_with_word_size<T>(args: &Args) -> Result<()>
where
    T: FileHeader<
        Word: PrimInt + WrappingSub + Integer + Serialize + PatchValue,
        Endian = Endianness,
    >,
{
    let loader_bytes = fs::read(&args.loader_path)?;

    let serialized_payload = serialize_payload::serialize_payload::<T>(
        &args.kernel_path,
        &args.app_path,
        &args.dtb_path,
        &args.platform_info_path,
    );

    let loader_with_payload_bytes = render_elf::render_elf::<T>(&loader_bytes, &serialized_payload);

    let out_file_path = &args.out_file_path;

    fs::write(out_file_path, loader_with_payload_bytes)?;
    Ok(())
}
