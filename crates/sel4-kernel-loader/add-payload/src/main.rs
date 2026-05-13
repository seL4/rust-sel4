//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs::{self, File};

use anyhow::Result;
use clap::Parser;
use object::elf::{FileHeader32, FileHeader64};
use object::read::elf::{ElfFile, FileHeader, ProgramHeader};
use object::{Endianness, ReadRef};
use rkyv::util::AlignedVec;

use sel4_config_types::Configuration;
use sel4_patch_elf::{FileHeaderExt, Patching};
use sel4_phdrs_constants::PT_SEL4_KERNEL_LOADER_PAYLOAD;
use sel4_platform_info_types::OwnedPlatformInfo;

mod maps;
mod page_tables;
mod serialize_payload;
mod utils;

use crate::page_tables::Scheme;
use crate::utils::{virt_footprint, with_elf};

type ArchiveAlignedVec = AlignedVec;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long)]
    sel4_prefix: Option<String>,
    #[arg(long)]
    sel4_config: Option<String>,
    #[arg(long)]
    kernel: Option<String>,
    #[arg(long)]
    dtb: Option<String>,
    #[arg(long)]
    platform_info: Option<String>,
    #[arg(long)]
    loader: String,
    #[arg(long)]
    app: String,
    #[arg(long, short = 'o')]
    out_file: String,
    #[arg(long, short = 'v')]
    verbose: bool,
}

#[derive(Debug)]
struct Paths {
    sel4_config_path: String,
    kernel_path: String,
    dtb_path: String,
    platform_info_path: String,
    loader_path: String,
    app_path: String,
    out_file_path: String,
}

impl Paths {
    fn get(cli: &Cli) -> Paths {
        let sel4_prefix = cli.sel4_prefix.as_ref();
        Paths {
            sel4_config_path: cli
                .sel4_config
                .as_ref()
                .map(ToOwned::to_owned)
                .or(sel4_prefix
                    .map(|prefix| format!("{prefix}/libsel4/include/kernel/gen_config.json")))
                .unwrap(),
            kernel_path: cli
                .kernel
                .as_ref()
                .map(ToOwned::to_owned)
                .or(sel4_prefix.map(|prefix| format!("{prefix}/bin/kernel.elf")))
                .unwrap(),
            dtb_path: cli
                .dtb
                .as_ref()
                .map(ToOwned::to_owned)
                .or(sel4_prefix.map(|prefix| format!("{prefix}/support/kernel.dtb")))
                .unwrap(),
            platform_info_path: cli
                .platform_info
                .as_ref()
                .map(ToOwned::to_owned)
                .or(sel4_prefix.map(|prefix| format!("{prefix}/support/platform_gen.yaml")))
                .unwrap(),
            loader_path: cli.loader.to_owned(),
            app_path: cli.app.to_owned(),
            out_file_path: cli.out_file.to_owned(),
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        eprintln!("{cli:#?}");
    }

    let paths = Paths::get(&cli);

    let kernel_config: Configuration =
        serde_json::from_reader(File::open(&paths.sel4_config_path).unwrap()).unwrap();

    match kernel_config.get("WORD_SIZE").unwrap().as_str().unwrap() {
        "32" => continue_with_type::<FileHeader32<Endianness>>(&paths, &kernel_config),
        "64" => continue_with_type::<FileHeader64<Endianness>>(&paths, &kernel_config),
        _ => {
            panic!()
        }
    }
}

fn continue_with_type<T>(paths: &Paths, kernel_config: &Configuration) -> Result<()>
where
    T: FileHeaderExt,
{
    let platform_info: OwnedPlatformInfo =
        serde_yaml::from_reader(fs::File::open(&paths.platform_info_path).unwrap()).unwrap();

    let payload = serialize_payload::serialize_payload::<T>(
        &paths.kernel_path,
        &paths.app_path,
        &paths.dtb_path,
        &platform_info,
    );

    let payload_data: AlignedVec = payload.to_bytes().unwrap();

    let orig_elf_bytes = fs::read(&paths.loader_path)?;
    let orig_elf = ElfFile::<T>::parse(&orig_elf_bytes).unwrap();

    let mut patching = Patching::new(&orig_elf);

    let scheme = Scheme::from_config(kernel_config);

    let smp = kernel_config
        .get("MAX_NUM_NODES")
        .unwrap()
        .as_str()
        .unwrap()
        .parse::<u32>()
        .unwrap()
        > 1;

    let min_level_align = 1
        << (0..scheme.num_levels())
            .map(|level| scheme.level_align_bits(level))
            .min()
            .unwrap();

    if kernel_config.get("ARCH_ARM").unwrap().as_bool().unwrap() {
        let mut addr_slot = None;
        patching.add_data_segment(min_level_align, |vaddr| {
            let (bytes, root_vaddr) = maps::mk_loader_map(&scheme, smp, vaddr, &platform_info);
            addr_slot = Some(root_vaddr);
            bytes
        });
        let addr = addr_slot.unwrap().try_into().unwrap();
        patching.patch_word("loader_level_0_table", addr);
    }

    {
        let mut addr_slot = None;
        patching.add_data_segment(min_level_align, |vaddr| {
            with_elf::<T, _, _>(&paths.kernel_path, |elf| {
                let phys_to_virt_offset = kernel_phys_to_virt_offset(elf, scheme.vaddr_mask());
                let virt_range = virt_footprint(elf);
                let masked_virt_addr_range =
                    virt_range.start & scheme.vaddr_mask()..virt_range.end & scheme.vaddr_mask();
                let (bytes, root_vaddr) = maps::mk_kernel_map(
                    &scheme,
                    smp,
                    vaddr,
                    masked_virt_addr_range,
                    phys_to_virt_offset,
                );
                addr_slot = Some(root_vaddr);
                bytes
            })
        });
        let addr = addr_slot.unwrap().try_into().unwrap();
        patching.patch_word("kernel_boot_level_0_table", addr);
    }

    patching.add_data_segment_with_meta_phdr(
        PT_SEL4_KERNEL_LOADER_PAYLOAD,
        ArchiveAlignedVec::ALIGNMENT.try_into().unwrap(),
        &payload_data,
    );

    fs::write(&paths.out_file_path, patching.finalize())?;
    Ok(())
}

fn kernel_phys_to_virt_offset<'a, T: FileHeader, R: ReadRef<'a>>(
    elf: &ElfFile<'a, T, R>,
    vaddr_mask: u64,
) -> u64 {
    let endian = elf.endian();
    let phdr = utils::loadable_segments(elf)
        .next()
        .unwrap()
        .elf_program_header();
    let vaddr = phdr.p_vaddr(endian).into() & vaddr_mask;
    let paddr = phdr.p_paddr(endian).into();
    vaddr.wrapping_sub(paddr)
}
