//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use num::NumCast;
use object::elf::PF_R;
use object::read::elf::FileHeader;

use sel4_patch_elf::{FileHeaderExt, GenericProgramHeader, Patching};
use sel4_phdrs_constants::{PT_SEL4_CAPDL_FRAME_DATA, PT_SEL4_CAPDL_SPEC};

// HACK
const PAGE_SIZE: u64 = 4096;

pub(crate) struct RenderElfArgs<'a> {
    pub(crate) spec_data: &'a [u8],
    pub(crate) spec_data_alignment: usize,
    pub(crate) embedded_frame_data: &'a [u8],
    pub(crate) embedded_frame_data_alignment: usize,
}

impl RenderElfArgs<'_> {
    pub(crate) fn call_with<T: FileHeader<Word: NumCast> + FileHeaderExt>(
        &self,
        orig_elf: &object::read::elf::ElfFile<T>,
    ) -> Vec<u8> {
        let mut patching = Patching::new(orig_elf);

        patching.add_segment_with_info_phdr(
            GenericProgramHeader {
                p_flags: PF_R,
                p_memsz: self.embedded_frame_data.len().try_into().unwrap(),
                p_align: PAGE_SIZE,
                ..Default::default()
            },
            self.embedded_frame_data_alignment.try_into().unwrap(),
            self.embedded_frame_data,
            PT_SEL4_CAPDL_FRAME_DATA,
        );

        patching.add_segment_with_info_phdr(
            GenericProgramHeader {
                p_flags: PF_R,
                p_memsz: self.spec_data.len().try_into().unwrap(),
                p_align: PAGE_SIZE,
                ..Default::default()
            },
            self.spec_data_alignment.try_into().unwrap(),
            self.spec_data,
            PT_SEL4_CAPDL_SPEC,
        );

        patching.finalize(PAGE_SIZE)
    }
}
