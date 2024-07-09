//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use num::NumCast;
use object::read::elf::FileHeader;

use sel4_synthetic_elf::{Builder, PatchValue, Segment};

pub fn render_elf<T>(orig_elf_buffer: &[u8], serialized_payload: &[u8]) -> Vec<u8>
where
    T: FileHeader<Word: NumCast + PatchValue>,
{
    let orig_elf_file = &object::read::elf::ElfFile::<T>::parse(orig_elf_buffer).unwrap();

    let mut builder = Builder::new(&orig_elf_file).unwrap();

    builder.discard_p_align(true);

    let vaddr = builder.next_vaddr().next_multiple_of(4096);

    builder.add_segment(Segment::simple(vaddr, serialized_payload.into()));

    builder
        .patch_word_with_cast("loader_payload_start", vaddr)
        .unwrap();
    builder
        .patch_word_with_cast("loader_payload_size", serialized_payload.len())
        .unwrap();
    builder
        .patch_word_with_cast("loader_image_start", builder.footprint().unwrap().start)
        .unwrap();
    builder
        .patch_word_with_cast("loader_image_end", builder.footprint().unwrap().end)
        .unwrap();

    builder.build().unwrap()
}
