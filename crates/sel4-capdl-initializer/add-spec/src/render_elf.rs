//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use num::NumCast;
use object::read::elf::{ElfFile, FileHeader};

use sel4_patch_elf::{FileHeaderExt, Patching};
use sel4_phdrs_constants::{PT_SEL4_CAPDL_FRAME_DATA, PT_SEL4_CAPDL_SPEC};

pub(crate) struct RenderElfArgs<'a> {
    pub(crate) spec_data: &'a [u8],
    pub(crate) spec_data_alignment: usize,
    pub(crate) embedded_frame_data: &'a [u8],
    pub(crate) embedded_frame_data_alignment: usize,
}

impl RenderElfArgs<'_> {
    pub(crate) fn call_with<T: FileHeader<Word: NumCast> + FileHeaderExt>(
        &self,
        orig_elf: &ElfFile<T>,
    ) -> Vec<u8> {
        let mut patching = Patching::new(orig_elf);

        patching.add_data_segment_with_meta_phdr(
            PT_SEL4_CAPDL_FRAME_DATA,
            self.embedded_frame_data_alignment.try_into().unwrap(),
            self.embedded_frame_data,
        );

        patching.add_data_segment_with_meta_phdr(
            PT_SEL4_CAPDL_SPEC,
            self.spec_data_alignment.try_into().unwrap(),
            self.spec_data,
        );

        patching.finalize()
    }
}
