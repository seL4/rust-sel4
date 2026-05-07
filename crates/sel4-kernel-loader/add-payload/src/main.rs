//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs::{self, File};

use anyhow::Result;
use num::{Integer, PrimInt, traits::WrappingSub};
use object::read::elf::ElfFile;
use object::{
    Endianness,
    elf::{FileHeader32, FileHeader64},
    read::elf::FileHeader,
};
use rkyv::util::AlignedVec;
use serde::Serialize;

use sel4_config_types::Configuration;
use sel4_patch_elf::{FileHeaderExt, Patching};
use sel4_phdrs_constants::PT_SEL4_KERNEL_LOADER_PAYLOAD;

mod args;
mod serialize_payload;

use args::Args;

type ArchiveAlignedVec = AlignedVec;

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
    T: FileHeader<Word: PrimInt + WrappingSub + Integer + Serialize, Endian = Endianness>
        + FileHeaderExt,
{
    let loader_bytes = fs::read(&args.loader_path)?;

    let payload = serialize_payload::serialize_payload::<T>(
        &args.kernel_path,
        &args.app_path,
        &args.dtb_path,
        &args.platform_info_path,
    );

    let payload_data: ArchiveAlignedVec = payload.to_bytes().unwrap();

    let loader_with_payload_bytes = {
        let orig_elf = ElfFile::<T>::parse(&loader_bytes).unwrap();
        let mut patching = Patching::new(&orig_elf);
        patching.add_data_segment(
            PT_SEL4_KERNEL_LOADER_PAYLOAD,
            ArchiveAlignedVec::ALIGNMENT.try_into().unwrap(),
            &payload_data,
        );
        patching.finalize()
    };

    let out_file_path = &args.out_file_path;

    fs::write(out_file_path, loader_with_payload_bytes)?;
    Ok(())
}
