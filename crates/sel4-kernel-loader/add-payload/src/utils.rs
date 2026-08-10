//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::{
    fs::File,
    ops::Range,
    path::Path,
};

use object::{
    Object,
    ObjectSegment,
    ReadCache,
    ReadRef,
    elf::PT_LOAD,
    read::elf::{
        ElfFile,
        ElfSegment,
        FileHeader,
        ProgramHeader,
    },
};

pub(crate) fn with_elf<T: FileHeader, R, F>(path: impl AsRef<Path>, f: F) -> R
where
    F: FnOnce(&ElfFile<T, &ReadCache<File>>) -> R,
{
    let file = File::open(path).unwrap();
    let read_cache = ReadCache::new(file);
    let elf = ElfFile::<T, _>::parse(&read_cache).unwrap();
    f(&elf)
}

pub(crate) fn loadable_segments<'data, 'file, T: FileHeader, R: ReadRef<'data>>(
    elf: &'file ElfFile<'data, T, R>,
) -> impl Iterator<Item = ElfSegment<'data, 'file, T, R>> {
    elf.segments()
        .filter(|seg| seg.elf_program_header().p_type(elf.endian()) == PT_LOAD)
}

pub(crate) fn virt_footprint<'a, T: FileHeader, R: ReadRef<'a>>(
    elf: &ElfFile<'a, T, R>,
) -> Range<u64> {
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
