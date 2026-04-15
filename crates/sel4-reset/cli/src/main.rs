//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs;

use anyhow::Result;
use num::NumCast;
use object::elf::{PF_W, PT_LOAD};
use object::read::elf::{ElfFile, FileHeader, ProgramHeader};
use object::{Endian, File, Object, ObjectSection, ReadCache, ReadRef};
use rangemap::RangeSet;

use sel4_synthetic_elf::{Builder, PatchValue, Segment, object};

mod args;

use args::Args;

fn main() -> Result<()> {
    let args = Args::parse()?;

    if args.verbose {
        eprintln!("{args:#?}");
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

fn continue_with_type<'a, T, R>(args: &Args, elf: &'a ElfFile<'a, T, R>) -> Result<()>
where
    R: ReadRef<'a>,
    T: FileHeader<Word: NumCast + PatchValue>,
{
    let endian = elf.endian();

    let persistent_ranges = {
        let mut set = RangeSet::new();
        for s in elf.sections() {
            if let Ok(name) = s.name()
                && (name == ".persistent" || name.starts_with(".persistent."))
            {
                set.insert(s.address()..(s.address() + s.size()));
            }
        }
        set
    };

    let mut builder = Builder::empty(elf);

    let mut regions_builder = RegionsBuilder::<T>::new();

    for phdr in elf.elf_program_headers() {
        if phdr.p_type(endian) == PT_LOAD {
            let segment = Segment::from_phdr(phdr, endian, elf.data())?;
            builder.add_segment(segment);
            if phdr.p_flags(endian) & PF_W != 0 {
                let vaddr = phdr.p_vaddr(endian).into();
                let memsz = phdr.p_memsz(endian).into();
                let data = phdr.data(endian, elf.data()).unwrap();
                let segment_range = vaddr..(vaddr + memsz);
                let relevant_persistent_ranges = RangeSet::from_iter(
                    persistent_ranges.intersection(&RangeSet::from_iter([segment_range.clone()])),
                );
                for ephermal in relevant_persistent_ranges.gaps(&segment_range) {
                    let region_memsz = ephermal.end - ephermal.start;
                    let region_offset_in_segment = ephermal.start - vaddr;
                    let region_data = if region_offset_in_segment < data.len().try_into().unwrap() {
                        &data[region_offset_in_segment.try_into().unwrap()
                            ..data.len().min((ephermal.end - vaddr).try_into().unwrap())]
                    } else {
                        &[]
                    };
                    regions_builder.add_region(ephermal.start, region_memsz, region_data);
                }
            }
        }
    }

    let regions = regions_builder.build(endian);

    let vaddr = builder.next_vaddr().next_multiple_of(4096);

    builder.add_segment(Segment::simple(vaddr, regions.raw.into()));

    builder
        .patch_word_with_cast("sel4_reset_regions_start", vaddr)
        .unwrap();
    builder
        .patch_word_with_cast("sel4_reset_regions_meta_offset", regions.meta_offset)
        .unwrap();
    builder
        .patch_word_with_cast("sel4_reset_regions_meta_count", regions.meta_count)
        .unwrap();
    builder
        .patch_word_with_cast("sel4_reset_regions_data_offset", regions.data_offset)
        .unwrap();
    builder
        .patch_word_with_cast("sel4_reset_regions_data_size", regions.data_size)
        .unwrap();

    builder.discard_p_align(true);

    let out_bytes = builder.build().unwrap();

    let out_file_path = &args.out_file_path;

    fs::write(out_file_path, out_bytes)?;
    Ok(())
}

struct RegionsBuilder<T: FileHeader> {
    meta: Vec<RegionMeta<T>>,
    data: Vec<u8>,
}

impl<T: FileHeader<Word: NumCast + PatchValue>> RegionsBuilder<T> {
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

struct RegionMeta<T: FileHeader> {
    vaddr: T::Word,
    offset: T::Word,
    filesz: T::Word,
    memsz: T::Word,
}

impl<T: FileHeader<Word: PatchValue>> RegionMeta<T> {
    fn pack(&self, endian: impl Endian, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.vaddr.to_bytes(endian));
        buf.extend_from_slice(&self.offset.to_bytes(endian));
        buf.extend_from_slice(&self.filesz.to_bytes(endian));
        buf.extend_from_slice(&self.memsz.to_bytes(endian));
    }
}

struct Regions {
    raw: Vec<u8>,
    meta_offset: usize,
    meta_count: usize,
    data_offset: usize,
    data_size: usize,
}
