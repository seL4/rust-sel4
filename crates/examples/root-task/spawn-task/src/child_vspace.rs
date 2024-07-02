//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::vec::Vec;
use core::ops::Range;

use object::{
    elf::{PF_R, PF_W, PF_X},
    Object, ObjectSegment, SegmentFlags,
};

use crate::ObjectAllocator;

const GRANULE_SIZE: usize = sel4::FrameObjectType::GRANULE.bytes();

pub(crate) fn create_child_vspace<'a>(
    allocator: &mut ObjectAllocator,
    image: &'a impl Object<'a>,
    caller_vspace: sel4::cap::VSpace,
    free_page_addr: usize,
    asid_pool: sel4::cap::AsidPool,
) -> (sel4::cap::VSpace, usize, sel4::cap::Granule) {
    let child_vspace = allocator.allocate_fixed_sized::<sel4::cap_type::VSpace>();
    asid_pool.asid_pool_assign(child_vspace).unwrap();

    let image_footprint = footprint(image);

    map_intermediate_translation_tables(
        allocator,
        child_vspace,
        image_footprint.start..(image_footprint.end + GRANULE_SIZE),
    );

    map_image(
        allocator,
        child_vspace,
        image_footprint.clone(),
        image,
        caller_vspace,
        free_page_addr,
    );

    let ipc_buffer_addr = image_footprint.end;
    let ipc_buffer_cap = allocator.allocate_fixed_sized::<sel4::cap_type::Granule>();
    ipc_buffer_cap
        .frame_map(
            child_vspace,
            ipc_buffer_addr,
            sel4::CapRights::read_write(),
            sel4::VmAttributes::default(),
        )
        .unwrap();

    (child_vspace, ipc_buffer_addr, ipc_buffer_cap)
}

fn footprint<'a>(image: &'a impl Object<'a>) -> Range<usize> {
    let min: usize = image
        .segments()
        .map(|seg| seg.address())
        .min()
        .unwrap()
        .try_into()
        .unwrap();
    let max: usize = image
        .segments()
        .map(|seg| seg.address() + seg.size())
        .max()
        .unwrap()
        .try_into()
        .unwrap();
    coarsen_footprint(&(min..max), GRANULE_SIZE)
}

fn map_intermediate_translation_tables(
    allocator: &mut ObjectAllocator,
    vspace: sel4::cap::VSpace,
    footprint: Range<usize>,
) {
    for level in 1..sel4::vspace_levels::NUM_LEVELS {
        let span_bytes = 1 << sel4::vspace_levels::span_bits(level);
        let footprint_at_level = coarsen_footprint(&footprint, span_bytes);
        for i in 0..(footprint_at_level.len() / span_bytes) {
            let ty = sel4::TranslationTableObjectType::from_level(level).unwrap();
            let addr = footprint_at_level.start + i * span_bytes;
            allocator
                .allocate(ty.blueprint())
                .cast::<sel4::cap_type::UnspecifiedIntermediateTranslationTable>()
                .generic_intermediate_translation_table_map(
                    ty,
                    vspace,
                    addr,
                    sel4::VmAttributes::default(),
                )
                .unwrap()
        }
    }
}

fn map_image<'a>(
    allocator: &mut ObjectAllocator,
    vspace: sel4::cap::VSpace,
    footprint: Range<usize>,
    image: &'a impl Object<'a>,
    caller_vspace: sel4::cap::VSpace,
    free_page_addr: usize,
) {
    let num_pages = footprint.len() / GRANULE_SIZE;
    let mut pages = (0..num_pages)
        .map(|_| {
            (
                allocator.allocate_fixed_sized::<sel4::cap_type::Granule>(),
                sel4::CapRightsBuilder::none(),
            )
        })
        .collect::<Vec<(sel4::cap::Granule, sel4::CapRightsBuilder)>>();

    for seg in image.segments() {
        let segment_addr = usize::try_from(seg.address()).unwrap();
        let segment_size = usize::try_from(seg.size()).unwrap();
        let segment_footprint =
            coarsen_footprint(&(segment_addr..(segment_addr + segment_size)), GRANULE_SIZE);
        let num_pages_spanned_by_segment = segment_footprint.len() / GRANULE_SIZE;
        let segment_data_size = seg.data().unwrap().len();
        let segment_data_footprint = coarsen_footprint(
            &(segment_addr..(segment_addr + segment_data_size)),
            GRANULE_SIZE,
        );
        let num_pages_spanned_by_segment_data = segment_data_footprint.len() / GRANULE_SIZE;

        let segment_page_index_offset = (segment_footprint.start - footprint.start) / GRANULE_SIZE;

        for (_, rights) in &mut pages[segment_page_index_offset..][..num_pages_spanned_by_segment] {
            add_rights(rights, seg.flags());
        }

        let mut data = seg.data().unwrap();
        let mut offset_into_page = segment_addr % GRANULE_SIZE;
        for (page_cap, _) in
            &pages[segment_page_index_offset..][..num_pages_spanned_by_segment_data]
        {
            let data_len = (GRANULE_SIZE - offset_into_page).min(data.len());

            page_cap
                .frame_map(
                    caller_vspace,
                    free_page_addr,
                    sel4::CapRights::read_write(),
                    sel4::VmAttributes::default(),
                )
                .unwrap();
            unsafe {
                ((free_page_addr + offset_into_page) as *mut u8).copy_from(data.as_ptr(), data_len);
            }
            page_cap.frame_unmap().unwrap();

            data = &data[data_len..];
            offset_into_page = 0;
        }
    }

    for (i, (page_cap, rights)) in pages.into_iter().enumerate() {
        let addr = footprint.start + i * GRANULE_SIZE;
        page_cap
            .frame_map(vspace, addr, rights.build(), sel4::VmAttributes::default())
            .unwrap();
    }
}

fn add_rights(rights: &mut sel4::CapRightsBuilder, flags: SegmentFlags) {
    match flags {
        SegmentFlags::Elf { p_flags } => {
            if p_flags & PF_R != 0 {
                *rights = rights.read(true);
            }
            if p_flags & PF_W != 0 {
                *rights = rights.write(true);
            }
            if p_flags & PF_X != 0 {
                *rights = rights.grant(true);
            }
        }
        _ => unimplemented!(),
    }
}

fn coarsen_footprint(footprint: &Range<usize>, granularity: usize) -> Range<usize> {
    round_down(footprint.start, granularity)..footprint.end.next_multiple_of(granularity)
}

const fn round_down(n: usize, b: usize) -> usize {
    n - n % b
}
