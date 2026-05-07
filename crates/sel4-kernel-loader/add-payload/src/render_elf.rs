//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use num::NumCast;
use object::read::elf::{ElfFile, FileHeader};

use sel4_patch_elf::{FileHeaderExt, Patching};
use sel4_phdrs_constants::PT_SEL4_KERNEL_LOADER_PAYLOAD;

pub fn render_elf<T>(orig_elf_buffer: &[u8], serialized_payload: &[u8]) -> Vec<u8>
where
    T: FileHeader<Word: NumCast> + FileHeaderExt,
{
    let orig_elf = ElfFile::<T>::parse(orig_elf_buffer).unwrap();
    let mut patching = Patching::new(&orig_elf);
    patching.add_data_segment(PT_SEL4_KERNEL_LOADER_PAYLOAD, 1, serialized_payload);
    patching.finalize()
}
