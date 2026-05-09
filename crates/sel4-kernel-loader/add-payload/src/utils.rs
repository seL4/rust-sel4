//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs::File;
use std::ops::Range;
use std::path::Path;

use object::elf::PT_LOAD;
use object::read::elf::{ElfFile, ElfSegment, FileHeader, ProgramHeader};
use object::{Object, ObjectSegment, ReadCache, ReadRef};

pub fn with_elf<T: FileHeader, R, F>(path: impl AsRef<Path>, f: F) -> R
where
    F: FnOnce(&ElfFile<T, &ReadCache<File>>) -> R,
{
    let file = File::open(path).unwrap();
    let read_cache = ReadCache::new(file);
    let elf = ElfFile::<T, _>::parse(&read_cache).unwrap();
    f(&elf)
}

pub fn loadable_segments<'data, 'file, T: FileHeader, R: ReadRef<'data>>(
    elf: &'file ElfFile<'data, T, R>,
) -> impl Iterator<Item = ElfSegment<'data, 'file, T, R>> {
    elf.segments()
        .filter(|seg| seg.elf_program_header().p_type(elf.endian()) == PT_LOAD)
}

pub fn virt_footprint<'a, T: FileHeader, R: ReadRef<'a>>(elf: &ElfFile<'a, T, R>) -> Range<u64> {
    let min = loadable_segments(elf)
        .map(|seg| seg.address())
        .min()
        .unwrap();
    let max = loadable_segments(elf)
        .map(|seg| seg.address().strict_add(seg.size()))
        .max()
        .unwrap();
    min..max
}
