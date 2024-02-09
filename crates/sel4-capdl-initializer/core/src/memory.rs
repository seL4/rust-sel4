//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ops::Range;

use sel4::SizedFrameType;

use super::frame_types;

#[repr(align(4096))]
struct SmallPagePlaceHolder(#[allow(dead_code)] [u8; frame_types::FrameType0::FRAME_SIZE.bytes()]);

static SMALL_PAGE_PLACEHOLDER: SmallPagePlaceHolder =
    SmallPagePlaceHolder([0; frame_types::FrameType0::FRAME_SIZE.bytes()]);

pub(crate) fn get_user_image_frame_slot(
    bootinfo: &sel4::BootInfoPtr,
    user_image_bounds: &Range<usize>,
    addr: usize,
) -> sel4::init_thread::Slot<frame_types::FrameType0> {
    assert_eq!(addr % frame_types::FrameType0::FRAME_SIZE.bytes(), 0);
    let num_user_frames = bootinfo.user_image_frames().len();
    let user_image_footprint = coarsen_footprint(
        user_image_bounds,
        frame_types::FrameType0::FRAME_SIZE.bytes(),
    );
    assert_eq!(
        user_image_footprint.len(),
        num_user_frames * frame_types::FrameType0::FRAME_SIZE.bytes()
    );
    let ix = (addr - user_image_footprint.start) / frame_types::FrameType0::FRAME_SIZE.bytes();
    bootinfo.user_image_frames().index(ix)
}

pub(crate) fn init_copy_addrs(
    bootinfo: &sel4::BootInfoPtr,
    user_image_bounds: &Range<usize>,
) -> Result<(usize, usize), sel4::Error> {
    let small_frame_copy_addr = {
        let addr = addr_of_ref(&SMALL_PAGE_PLACEHOLDER);
        let slot = get_user_image_frame_slot(bootinfo, user_image_bounds, addr);
        let cap = slot.local_cptr();
        cap.frame_unmap()?;
        addr
    };
    let large_frame_copy_addr = {
        let addr_space_footprint = coarsen_footprint(
            &(user_image_bounds.start..(user_image_bounds.start + bootinfo.footprint_size())),
            frame_types::FrameType1::FRAME_SIZE.bytes(),
        );
        match (
            addr_space_footprint.start % frame_types::FrameType2::FRAME_SIZE.bytes(),
            addr_space_footprint.end % frame_types::FrameType2::FRAME_SIZE.bytes(),
        ) {
            (0, 0) => panic!(), // absurd
            (_, 0) => {
                round_down(
                    addr_space_footprint.start,
                    frame_types::FrameType1::FRAME_SIZE.bytes(),
                ) - frame_types::FrameType1::FRAME_SIZE.bytes()
            }
            (_, _) => addr_space_footprint
                .end
                .next_multiple_of(frame_types::FrameType1::FRAME_SIZE.bytes()),
        }
    };
    Ok((small_frame_copy_addr, large_frame_copy_addr))
}

fn coarsen_footprint(footprint: &Range<usize>, granularity: usize) -> Range<usize> {
    round_down(footprint.start, granularity)..footprint.end.next_multiple_of(granularity)
}

const fn round_down(n: usize, b: usize) -> usize {
    n - n % b
}

fn addr_of_ref<T>(x: &T) -> usize {
    x as *const T as usize
}
