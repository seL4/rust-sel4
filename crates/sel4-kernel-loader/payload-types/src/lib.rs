//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::cmp;
use core::fmt;
use core::ops::Range;
use core::ptr;
use core::slice;

use rkyv::Archive;
use rkyv::rancor;
use rkyv::util::AlignedVec;

use sel4_platform_info_types::PlatformInfo;

#[derive(Debug, Copy, Clone, Eq, PartialEq, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
#[rkyv(derive(Copy, Clone, Eq, PartialEq))]
pub struct Word(pub u64);

impl ArchivedWord {
    pub fn to_usize(&self) -> usize {
        self.0.try_into().unwrap()
    }
}

impl fmt::Debug for ArchivedWord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(rkyv::Archive, rkyv::Serialize)]
pub struct Payload {
    pub info: PayloadInfo,
    pub data: Vec<Region>,
}

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize)]
#[rkyv(derive(Debug, Clone))]
pub struct PayloadInfo {
    pub kernel_entry: Word,
    pub user_image: UserImageInfo,
    pub dtb: Option<DtbInfo>,
}

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize)]
#[rkyv(derive(Debug, Clone))]
pub struct UserImageInfo {
    pub ui_p_reg_start: Word,
    pub ui_p_reg_end: Word,
    pub pv_offset: Word,
    pub v_entry: Word,
}

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize)]
#[rkyv(derive(Debug, Clone))]
pub struct DtbInfo {
    pub addr_p: Word,
    pub size: Word,
}

#[derive(Debug, rkyv::Archive, rkyv::Serialize)]
pub struct Region {
    pub addr: Word,
    pub size: Word,
    pub data: Vec<u8>,
}

impl ArchivedRegion {
    fn addr_range(&self) -> Range<usize> {
        self.addr.to_usize()..self.addr.to_usize().strict_add(self.size.to_usize())
    }
}

impl Payload {
    pub fn to_bytes(&self) -> Result<AlignedVec, rancor::Error> {
        rkyv::to_bytes(self)
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn access_unchecked(buf: &[u8]) -> &<Self as Archive>::Archived {
        unsafe { rkyv::access_unchecked(buf) }
    }
}

impl ArchivedPayload {
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn copy_data_out(&self) {
        for region in self.data.iter() {
            let src = &region.data;
            let dst = unsafe {
                slice::from_raw_parts_mut(region.addr.to_usize() as *mut _, region.size.to_usize())
            };
            let (dst_data, dst_zero) = dst.split_at_mut(src.len());
            dst_data.copy_from_slice(src);
            // NOTE slice::fill is too slow
            // TODO(nspin) is that still true?
            unsafe {
                ptr::write_bytes(dst_zero.as_mut_ptr(), 0, dst_zero.len());
            }
        }
    }

    pub fn sanity_check<T: TryFrom<usize, Error: fmt::Debug> + Ord>(
        &self,
        platform_info: &PlatformInfo<T>,
        own_footprint: Range<usize>,
    ) {
        let memory = &platform_info.memory;
        assert!(any_range_contains(
            memory.iter(),
            &range_try_into(&own_footprint).unwrap()
        ));
        for region in self.data.iter() {
            assert!(any_range_contains(
                memory.iter(),
                &range_try_into(&region.addr_range()).unwrap()
            ));
            assert!(ranges_are_disjoint(
                &own_footprint,
                &range_try_into(&region.addr_range()).unwrap()
            ));
        }
    }
}

fn ranges_are_disjoint<T: Ord>(this: &Range<T>, that: &Range<T>) -> bool {
    cmp::min(&this.end, &that.end) <= cmp::max(&this.start, &that.start)
}

fn range_contains<T: Ord>(this: &Range<T>, that: &Range<T>) -> bool {
    this.start <= that.start && that.end <= this.end
}

fn range_try_into<T: TryInto<U, Error: fmt::Debug> + Copy, U>(
    this: &Range<T>,
) -> Result<Range<U>, T::Error> {
    Ok(this.start.try_into()?..this.end.try_into()?)
}

fn any_range_contains<'a, T: Ord + 'a>(
    mut these: impl Iterator<Item = &'a Range<T>>,
    that: &Range<T>,
) -> bool {
    these.any(|this| range_contains(this, that))
}
