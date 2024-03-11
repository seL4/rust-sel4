//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ops::Range;
use core::ptr;

use sel4::{cap_type, init_thread, sel4_cfg_attr, sel4_cfg_bool, CapTypeForFrameObjectOfFixedSize};

const SMALL_PAGE_PLACEHOLDER_SIZE: usize = if sel4_cfg_bool!(ARCH_AARCH32) {
    65536
} else {
    4096
};

#[sel4_cfg_attr(ARCH_AARCH32, repr(align(65536)))]
#[sel4_cfg_attr(not(ARCH_AARCH32), repr(align(4096)))]
struct SmallPagePlaceHolder(#[allow(dead_code)] [u8; SMALL_PAGE_PLACEHOLDER_SIZE]);

static SMALL_PAGE_PLACEHOLDER: SmallPagePlaceHolder =
    SmallPagePlaceHolder([0; SMALL_PAGE_PLACEHOLDER_SIZE]);

pub(crate) struct CopyAddrs {
    smaller_frame_copy_addr: usize,
    larger_frame_copy_addr: usize,
}

impl CopyAddrs {
    pub(crate) fn init(
        bootinfo: &sel4::BootInfoPtr,
        user_image_bounds: &Range<usize>,
    ) -> Result<Self, sel4::Error> {
        let smaller_frame_copy_addr = {
            let addr = ptr::addr_of!(SMALL_PAGE_PLACEHOLDER) as usize;
            let start_slot_index =
                get_user_image_frame_slot(bootinfo, user_image_bounds, addr).index();
            for i in 0..(SMALL_PAGE_PLACEHOLDER_SIZE / cap_type::Granule::FRAME_OBJECT_TYPE.bytes())
            {
                let slot = init_thread::Slot::<cap_type::Granule>::from_index(start_slot_index + i);
                let cap = slot.cap();
                cap.frame_unmap()?;
            }
            addr
        };
        let larger_frame_copy_addr = {
            let outer_span = 1u64
                << sel4::TranslationStructureObjectType::span_bits(
                    sel4::TranslationStructureObjectType::NUM_LEVELS - 2,
                );
            let inner_span = 1usize
                << sel4::TranslationStructureObjectType::span_bits(
                    sel4::TranslationStructureObjectType::NUM_LEVELS - 1,
                );
            let addr_space_footprint = coarsen_footprint(
                &(user_image_bounds.start..(user_image_bounds.start + bootinfo.footprint_size())),
                inner_span,
            );
            match (
                u64::try_from(addr_space_footprint.start).unwrap() % outer_span,
                u64::try_from(addr_space_footprint.end).unwrap() % outer_span,
            ) {
                (0, 0) => panic!(), // absurd
                (_, 0) => round_down(addr_space_footprint.start, inner_span) - inner_span,
                (_, _) => addr_space_footprint.end.next_multiple_of(inner_span),
            }
        };
        Ok(Self {
            smaller_frame_copy_addr,
            larger_frame_copy_addr,
        })
    }

    pub(crate) fn select(&self, frame_object_type: sel4::FrameObjectType) -> usize {
        if frame_object_type.bytes() <= SMALL_PAGE_PLACEHOLDER_SIZE {
            self.smaller_frame_copy_addr
        } else {
            assert_eq!(
                frame_object_type.bits(),
                sel4::TranslationStructureObjectType::span_bits(
                    sel4::TranslationStructureObjectType::NUM_LEVELS - 1
                )
            );
            self.larger_frame_copy_addr
        }
    }
}

pub(crate) fn get_user_image_frame_slot(
    bootinfo: &sel4::BootInfoPtr,
    user_image_bounds: &Range<usize>,
    addr: usize,
) -> init_thread::Slot<cap_type::Granule> {
    let granule_size = cap_type::Granule::FRAME_OBJECT_TYPE.bytes();
    assert_eq!(addr % granule_size, 0);
    let num_user_frames = bootinfo.user_image_frames().len();
    let user_image_footprint = coarsen_footprint(user_image_bounds, granule_size);
    assert_eq!(user_image_footprint.len(), num_user_frames * granule_size);
    let ix = (addr - user_image_footprint.start) / granule_size;
    bootinfo.user_image_frames().index(ix)
}

fn coarsen_footprint(footprint: &Range<usize>, granularity: usize) -> Range<usize> {
    round_down(footprint.start, granularity)..footprint.end.next_multiple_of(granularity)
}

const fn round_down(n: usize, b: usize) -> usize {
    n - n % b
}
