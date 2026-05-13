//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs::{self, File};

use anyhow::Result;
use object::elf::{FileHeader32, FileHeader64};
use object::read::elf::{ElfFile, FileHeader, ProgramHeader};
use object::{Endianness, ReadRef};
use rkyv::util::AlignedVec;

use sel4_config_types::Configuration;
use sel4_patch_elf::{FileHeaderExt, Patching};
use sel4_phdrs_constants::PT_SEL4_KERNEL_LOADER_PAYLOAD;
use sel4_platform_info_types::OwnedPlatformInfo;

mod args;
mod maps;
mod page_tables;
mod serialize_payload;
mod utils;

use args::Args;

use crate::page_tables::Scheme;
use crate::utils::{virt_footprint, with_elf};

type ArchiveAlignedVec = AlignedVec;

fn main() -> Result<()> {
    let args = Args::parse()?;

    if args.verbose {
        eprintln!("{args:#?}");
    }

    let kernel_config: Configuration =
        serde_json::from_reader(File::open(&args.sel4_config_path).unwrap()).unwrap();

    match kernel_config.get("WORD_SIZE").unwrap().as_str().unwrap() {
        "32" => continue_with_type::<FileHeader32<Endianness>>(&args, &kernel_config),
        "64" => continue_with_type::<FileHeader64<Endianness>>(&args, &kernel_config),
        _ => {
            panic!()
        }
    }
}

fn continue_with_type<T>(args: &Args, kernel_config: &Configuration) -> Result<()>
where
    T: FileHeaderExt,
{
    let platform_info: OwnedPlatformInfo =
        serde_yaml::from_reader(fs::File::open(&args.platform_info_path).unwrap()).unwrap();

    let payload = serialize_payload::serialize_payload::<T>(
        &args.kernel_path,
        &args.app_path,
        &args.dtb_path,
        &platform_info,
    );

    let payload_data: ArchiveAlignedVec = payload.to_bytes().unwrap();

    let orig_elf_bytes = fs::read(&args.loader_path)?;
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
            with_elf::<T, _, _>(&args.kernel_path, |elf| {
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

    fs::write(&args.out_file_path, patching.finalize())?;
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
