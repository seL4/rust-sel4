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
use rkyv::ops::ArchivedRange;
use rkyv::rancor;
use rkyv::util::AlignedVec;

use sel4_platform_info_types::PlatformInfo;

#[derive(Debug, Copy, Clone, Eq, PartialEq, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
#[rkyv(derive(Debug, Copy, Clone, Eq, PartialEq))]
pub struct Word(pub u64);

impl From<u64> for Word {
    fn from(x: u64) -> Word {
        Word(x)
    }
}

impl From<Word> for u64 {
    fn from(x: Word) -> u64 {
        x.0
    }
}

impl Word {
    pub fn from_u64(x: impl Into<u64>) -> Self {
        x.into().into()
    }

    pub fn from_u64_range(range: &Range<impl Into<u64> + Copy>) -> Range<Self> {
        Self::from_u64(range.start)..Self::from_u64(range.end)
    }
}

impl ArchivedWord {
    pub fn to_usize(&self) -> usize {
        self.0.try_into().unwrap()
    }

    pub fn to_usize_range(range: &ArchivedRange<Self>) -> Range<usize> {
        range.start.to_usize()..range.end.to_usize()
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
    pub kernel_image: ImageInfo,
    pub user_image: ImageInfo,
    pub fdt_phys_addr_range: Option<Range<Word>>,
}

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize)]
#[rkyv(derive(Debug, Clone))]
pub struct ImageInfo {
    pub phys_addr_range: Range<Word>,
    // TODO invert and i64
    pub phys_to_virt_offset: Word,
    pub virt_entry: Word,
}

// impl ImageInfo {
//     pub fn virt_addr_range(&self) -> Range<Word> {
//         self.phys_to_virt(self.phys_addr_range.start)..self.phys_to_virt(self.phys_addr_range.end)
//     }

//     pub fn phys_to_virt(&self, paddr: Word) -> Word {
//         // TODO must parameterize over word size for wrapping
//         Word(paddr.0.wrapping_add(self.phys_to_virt_offset.0))
//     }
// }

#[derive(Debug, rkyv::Archive, rkyv::Serialize)]
pub struct Region {
    pub phys_addr_range: Range<Word>,
    // TODO no Option
    pub content: Option<Vec<u8>>,
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
            let dst = unsafe {
                slice::from_raw_parts_mut(
                    region.phys_addr_range.start.to_usize() as *mut _,
                    region.phys_addr_range.end.to_usize() - region.phys_addr_range.start.to_usize(),
                )
            };
            match region.content.as_ref() {
                Some(src) => {
                    dst.copy_from_slice(src);
                }
                None => {
                    // NOTE slice::fill is too slow
                    // TODO(nspin) is that still true?
                    unsafe {
                        ptr::write_bytes(dst.as_mut_ptr(), 0, dst.len());
                    }
                }
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
                &range_try_into(&ArchivedWord::to_usize_range(&region.phys_addr_range)).unwrap()
            ));
            assert!(ranges_are_disjoint(
                &own_footprint,
                &ArchivedWord::to_usize_range(&region.phys_addr_range)
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
