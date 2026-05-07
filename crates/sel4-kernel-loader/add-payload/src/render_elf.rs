//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use num::NumCast;
use object::elf::PF_R;
use object::read::elf::FileHeader;

use sel4_patch_elf::{FileHeaderExt, GenericProgramHeader, Patching};

const PT_SEL4_KERNEL_LOADER_PAYLOAD: u32 = 0x64c3_4004;

// HACK
const PAGE_SIZE: u64 = 4096;

pub fn render_elf<T>(orig_elf_buffer: &[u8], serialized_payload: &[u8]) -> Vec<u8>
where
    T: FileHeader<Word: NumCast> + FileHeaderExt,
{
    let orig_elf_file = &object::read::elf::ElfFile::<T>::parse(orig_elf_buffer).unwrap();

    let mut patching = Patching::new(orig_elf_file);

    patching.add_segment_with_info_phdr(
        GenericProgramHeader {
            p_flags: PF_R,
            p_memsz: serialized_payload.len().try_into().unwrap(),
            p_align: PAGE_SIZE,
            ..Default::default()
        },
        PAGE_SIZE,
        serialized_payload,
        PT_SEL4_KERNEL_LOADER_PAYLOAD,
    );

    patching.finalize(PAGE_SIZE)
}
