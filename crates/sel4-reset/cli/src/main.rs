//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs;
use std::mem;

use anyhow::Result;
use num::{NumCast, Zero};
use object::elf::{PF_W, PT_LOAD};
use object::read::elf::{ElfFile, ProgramHeader};
use object::{Endian, File, Object, ObjectSection, ReadCache, ReadRef};

use sel4_render_elf_with_data::{FileHeaderExt, Input, SymbolicInjection, SymbolicValue};

mod args;

use args::Args;

fn main() -> Result<()> {
    let args = Args::parse()?;

    if args.verbose {
        eprintln!("{:#?}", args);
    }

    let in_file = fs::File::open(&args.in_file_path)?;
    let in_file_cached = ReadCache::new(in_file);
    let in_obj_file = File::parse(&in_file_cached)?;

    match in_obj_file {
        File::Elf32(elf) => continue_with_type(&args, &elf),
        File::Elf64(elf) => continue_with_type(&args, &elf),
        _ => {
            panic!()
        }
    }
}

fn continue_with_type<'a, T, R>(args: &Args, elf: &ElfFile<'a, T, R>) -> Result<()>
where
    R: ReadRef<'a>,
    T: FileHeaderExt,
{
    let endian = elf.endian();

    let persistent_section = elf.section_by_name(".persistent");

    let mut regions_builder = RegionsBuilder::<T>::new();

    let mut persistent_section_placed = false;

    for phdr in elf.elf_program_headers().iter() {
        if phdr.p_type(endian) == PT_LOAD && phdr.p_flags(endian) & PF_W != 0 {
            let vaddr = <u64 as NumCast>::from(phdr.p_vaddr(endian)).unwrap();
            let memsz = <u64 as NumCast>::from(phdr.p_memsz(endian)).unwrap();
            let data = phdr.data(endian, elf.data()).unwrap();
            let skip = persistent_section
                .as_ref()
                .and_then(|section| {
                    if section.address() == vaddr {
                        persistent_section_placed = true;
                        Some(section.size())
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            regions_builder.add_region_with_skip(skip, vaddr, memsz, data);
        }
    }

    assert!(!(persistent_section.is_some() && !persistent_section_placed));

    let out_bytes = regions_builder
        .build(endian)
        .input::<T>()
        .render_with_data_already_parsed(elf)
        .unwrap();

    let out_file_path = &args.out_file_path;

    fs::write(out_file_path, out_bytes)?;
    Ok(())
}

struct RegionsBuilder<T: FileHeaderExt> {
    meta: Vec<RegionMeta<T>>,
    data: Vec<u8>,
}

impl<T: FileHeaderExt> RegionsBuilder<T> {
    fn new() -> Self {
        Self {
            meta: vec![],
            data: vec![],
        }
    }

    fn add_region(&mut self, vaddr: u64, memsz: u64, data: &[u8]) {
        let offset = self.data.len();
        let filesz = data.len();
        self.data.extend_from_slice(data);
        self.meta.push(RegionMeta {
            vaddr: NumCast::from(vaddr).unwrap(),
            offset: NumCast::from(offset).unwrap(),
            filesz: NumCast::from(filesz).unwrap(),
            memsz: NumCast::from(memsz).unwrap(),
        })
    }

    fn add_region_with_skip(&mut self, skip: u64, vaddr: u64, memsz: u64, data: &[u8]) {
        if skip < memsz {
            self.add_region(
                vaddr + skip,
                memsz - skip,
                &data[data.len().min(skip.try_into().unwrap())..],
            );
        }
    }

    fn build(&self, endian: impl Endian) -> Regions {
        let mut raw = vec![];
        let meta_offset = raw.len();
        let meta_count = self.meta.len();
        for meta in self.meta.iter() {
            meta.pack(endian, &mut raw);
        }
        let data_offset = raw.len();
        let data_size = self.data.len();
        raw.extend_from_slice(&self.data);
        Regions {
            raw,
            meta_offset,
            meta_count,
            data_offset,
            data_size,
        }
    }
}

struct RegionMeta<T: FileHeaderExt> {
    vaddr: T::Word,
    offset: T::Word,
    filesz: T::Word,
    memsz: T::Word,
}

impl<T: FileHeaderExt> RegionMeta<T> {
    fn pack(&self, endian: impl Endian, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&T::write_word_bytes(endian, self.vaddr));
        buf.extend_from_slice(&T::write_word_bytes(endian, self.offset));
        buf.extend_from_slice(&T::write_word_bytes(endian, self.filesz));
        buf.extend_from_slice(&T::write_word_bytes(endian, self.memsz));
    }
}

struct Regions {
    raw: Vec<u8>,
    meta_offset: usize,
    meta_count: usize,
    data_offset: usize,
    data_size: usize,
}

impl Regions {
    fn input<T: FileHeaderExt>(&self) -> Input<T> {
        let align_modulus = NumCast::from(4096).unwrap();
        let align_residue = T::Word::zero();
        let memsz = self.raw.len();
        let mut input = Input::<T>::default();
        input.symbolic_injections.push(SymbolicInjection {
            align_modulus,
            align_residue,
            content: &self.raw,
            memsz: NumCast::from(memsz).unwrap(),
            patches: vec![(
                "sel4_reset_regions_start".to_owned(),
                SymbolicValue {
                    addend: T::Sword::zero(),
                },
            )],
        });
        input.concrete_patches.push((
            "sel4_reset_regions_meta_offset".to_owned(),
            NumCast::from(self.meta_offset).unwrap(),
        ));
        input.concrete_patches.push((
            "sel4_reset_regions_meta_count".to_owned(),
            NumCast::from(self.meta_count).unwrap(),
        ));
        input.concrete_patches.push((
            "sel4_reset_regions_data_offset".to_owned(),
            NumCast::from(self.data_offset).unwrap(),
        ));
        input.concrete_patches.push((
            "sel4_reset_regions_data_size".to_owned(),
            NumCast::from(self.data_size).unwrap(),
        ));
        input
    }
}
