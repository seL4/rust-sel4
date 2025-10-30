//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use num::NumCast;
use object::read::elf::FileHeader;

use sel4_synthetic_elf::{Builder, PatchValue, Segment};

pub(crate) struct RenderElfArgs<'a> {
    pub(crate) spec_data: &'a [u8],
    pub(crate) spec_data_alignment: usize,
    pub(crate) embedded_frame_data: &'a [u8],
    pub(crate) embedded_frame_data_alignment: usize,
}

impl RenderElfArgs<'_> {
    pub(crate) fn call_with<T: FileHeader<Word: NumCast + PatchValue>>(
        &self,
        orig_elf: &object::read::elf::ElfFile<T>,
    ) -> Vec<u8> {
        let mut builder = Builder::new(orig_elf).unwrap();
        builder.discard_p_align(true);

        let embedded_frame_data_vaddr = builder
            .next_vaddr()
            .next_multiple_of(self.embedded_frame_data_alignment.try_into().unwrap());
        builder.add_segment(Segment::simple(
            embedded_frame_data_vaddr,
            self.embedded_frame_data.into(),
        ));
        builder
            .patch_word_with_cast(
                "sel4_capdl_initializer_embedded_frames_data_start",
                embedded_frame_data_vaddr,
            )
            .unwrap();

        let serialized_spec_data_vaddr = builder
            .next_vaddr()
            .next_multiple_of(self.spec_data_alignment.try_into().unwrap());
        builder.add_segment(Segment::simple(
            serialized_spec_data_vaddr,
            self.spec_data.into(),
        ));
        builder
            .patch_word_with_cast(
                "sel4_capdl_initializer_serialized_spec_data_start",
                serialized_spec_data_vaddr,
            )
            .unwrap();
        builder
            .patch_word_with_cast(
                "sel4_capdl_initializer_serialized_spec_data_size",
                self.spec_data.len(),
            )
            .unwrap();

        builder
            .patch_word_with_cast(
                "sel4_capdl_initializer_image_start",
                builder.footprint().unwrap().start,
            )
            .unwrap();
        builder
            .patch_word_with_cast(
                "sel4_capdl_initializer_image_end",
                builder.footprint().unwrap().end,
            )
            .unwrap();

        builder.build().unwrap()
    }
}
