//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs;
use std::path::PathBuf;

use anyhow::Error;
use clap::Parser;
use num::NumCast;
use object::elf::{PF_R, PF_W, PT_LOAD};
use object::read::elf::{ElfFile, FileHeader, ProgramHeader};
use object::{File, Object, ObjectSection, Pod, pod};
use rangemap::RangeSet;

mod patch;

use patch::{GenericProgramHeader, Patching, ProgramHeaderExt};

pub const PT_SEL4_RESET_REGIONS: u32 = 0x64c3_4001;

// HACK
const PAGE_SIZE: u64 = 4096;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    in_file_path: PathBuf,
    #[arg(short = 'o')]
    out_file_path: PathBuf,
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let in_perms = fs::metadata(&cli.in_file_path)?.permissions();
    let in_bytes = fs::read(&cli.in_file_path)?;
    let in_file = File::parse(in_bytes.as_slice())?;

    let out_bytes = match in_file {
        File::Elf32(elf) => continue_with_type(&elf),
        File::Elf64(elf) => continue_with_type(&elf),
        _ => {
            panic!()
        }
    }?;

    fs::write(&cli.out_file_path, &out_bytes)?;
    fs::set_permissions(&cli.out_file_path, in_perms)?;

    Ok(())
}

fn continue_with_type<'a, T>(orig_elf: &'a ElfFile<'a, T>) -> Result<Vec<u8>, Error>
where
    T: FileHeader<Word: NumCast + Pod, ProgramHeader: ProgramHeaderExt>,
{
    let mut patching = Patching::new(orig_elf);
    add_regions(&mut patching)?;
    Ok(patching.finalize())
}

struct RegionMeta<T: FileHeader> {
    dst_vaddr: T::Word,
    dst_size: T::Word,
    src_vaddr: T::Word,
    src_size: T::Word,
}

impl<T: FileHeader<Word: Pod>> RegionMeta<T> {
    fn pack(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(pod::bytes_of(&self.dst_vaddr));
        buf.extend_from_slice(pod::bytes_of(&self.dst_size));
        buf.extend_from_slice(pod::bytes_of(&self.src_vaddr));
        buf.extend_from_slice(pod::bytes_of(&self.src_size));
    }

    fn pack_to_vec(regions: &[RegionMeta<T>]) -> Vec<u8> {
        let mut buf = vec![];
        for meta in regions.iter() {
            meta.pack(&mut buf);
        }
        buf
    }
}

fn get_all_persistent_ranges<'a, T>(elf: &'a ElfFile<'a, T>) -> RangeSet<u64>
where
    T: FileHeader<Word: NumCast + Pod, ProgramHeader: ProgramHeaderExt>,
{
    let mut set = RangeSet::new();
    for s in elf.sections() {
        if let Ok(name) = s.name()
            && (name == ".persistent" || name.starts_with(".persistent."))
        {
            set.insert(s.address()..(s.address() + s.size()));
        }
    }
    set
}

fn add_regions<'a, T: FileHeader<Word: Pod + NumCast, ProgramHeader: ProgramHeaderExt>>(
    this: &mut Patching<'a, T>,
) -> Result<(), Error> {
    let endian = this.endian();

    let mut regions: Vec<RegionMeta<T>> = vec![];
    for seg in this.orig_elf().segments() {
        let phdr = seg.elf_program_header();
        if phdr.p_type(endian) == PT_LOAD && phdr.p_flags(endian) & PF_W != 0 {
            let p_offset = phdr.p_offset(endian).into();
            let p_vaddr = phdr.p_vaddr(endian).into();
            let p_memsz = phdr.p_memsz(endian).into();
            let p_filesz = phdr.p_filesz(endian).into();
            let p_align = phdr.p_align(endian).into();
            let ro_phdr = this.add_segment_raw(GenericProgramHeader {
                p_type: PT_LOAD,
                p_flags: PF_R,
                p_offset,
                p_vaddr: 0,
                p_paddr: 0,
                p_filesz,
                p_memsz: p_filesz,
                p_align,
            })?;
            let ro_p_vaddr = ro_phdr.p_vaddr(endian).into();
            let ro_p_filesz = ro_phdr.p_filesz(endian).into();
            for ephermal_region in
                get_all_persistent_ranges(this.orig_elf()).gaps(&(p_vaddr..(p_vaddr + p_memsz)))
            {
                let ephermal_region_size = ephermal_region.end - ephermal_region.start;
                let ephermal_region_offset_in_segment = ephermal_region.start - p_vaddr;
                let (src_vaddr, src_size) = if ephermal_region_offset_in_segment < ro_p_filesz {
                    (
                        ro_p_vaddr + ephermal_region_offset_in_segment,
                        (ro_p_filesz - ephermal_region_offset_in_segment).min(ephermal_region_size),
                    )
                } else {
                    (0, 0)
                };
                regions.push(RegionMeta {
                    dst_vaddr: <T::Word as NumCast>::from(ephermal_region.start).unwrap(),
                    dst_size: <T::Word as NumCast>::from(ephermal_region_size).unwrap(),
                    src_vaddr: <T::Word as NumCast>::from(src_vaddr).unwrap(),
                    src_size: <T::Word as NumCast>::from(src_size).unwrap(),
                });
            }
        }
    }

    let regions_meta_data = RegionMeta::pack_to_vec(&regions);
    let mut regions_meta_info_phdr = *this.add_segment(
        GenericProgramHeader {
            p_type: PT_LOAD,
            p_flags: PF_R,
            p_memsz: regions_meta_data.len().try_into().unwrap(),
            p_align: PAGE_SIZE,
            ..Default::default()
        },
        align_of::<RegionMeta<T>>().try_into().unwrap(),
        &regions_meta_data,
    )?;
    regions_meta_info_phdr.set_p_type(endian, PT_SEL4_RESET_REGIONS);
    this.add_phdr(regions_meta_info_phdr)?;
    Ok(())
}
