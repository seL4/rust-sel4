//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(strict_provenance)]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(clippy::useless_conversion)]

use core::ops::Range;
use core::ptr;
use core::slice;

use heapless::Vec;
use num_traits::{PrimInt, WrappingAdd};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use sel4_platform_info_types::PlatformInfo;

pub const DEFAULT_MAX_NUM_REGIONS: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Payload<T, U = IndirectRegionContent<T>, const N: usize = DEFAULT_MAX_NUM_REGIONS> {
    pub info: PayloadInfo<T>,
    pub data: Vec<Region<T, U>, N>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PayloadInfo<T> {
    pub kernel_image: ImageInfo<T>,
    pub user_image: ImageInfo<T>,
    pub fdt_phys_addr_range: Option<Range<T>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ImageInfo<T> {
    pub phys_addr_range: Range<T>,
    pub phys_to_virt_offset: T,
    pub virt_entry: T,
}

impl<T: PrimInt + WrappingAdd> ImageInfo<T> {
    pub fn virt_addr_range(&self) -> Range<T> {
        self.phys_to_virt(self.phys_addr_range.start)..self.phys_to_virt(self.phys_addr_range.end)
    }

    pub fn phys_to_virt(&self, paddr: T) -> T {
        paddr.wrapping_add(&self.phys_to_virt_offset)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Region<T, U> {
    pub phys_addr_range: Range<T>,
    pub content: Option<U>,
}

impl<T: Clone, U> Region<T, U> {
    pub fn traverse<V, E>(&self, mut f: impl FnMut(&U) -> Result<V, E>) -> Result<Region<T, V>, E> {
        Ok(Region {
            phys_addr_range: self.phys_addr_range.clone(),
            content: self.content.as_ref().map(&mut f).transpose()?,
        })
    }
}

#[allow(clippy::len_without_is_empty)]
pub trait RegionContent {
    type Source: ?Sized;

    fn len(&self) -> usize;

    fn copy_out(&self, source: &Self::Source, dst: &mut [u8]);
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IndirectRegionContent<T> {
    pub content_range: Range<T>,
}

impl<T: PrimInt> IndirectRegionContent<T> {
    fn to_usize_range(&self) -> Range<usize> {
        self.content_range.start.to_usize().unwrap()..self.content_range.end.to_usize().unwrap()
    }
}

impl<T: PrimInt> RegionContent for IndirectRegionContent<T> {
    type Source = [u8];

    fn len(&self) -> usize {
        self.to_usize_range().len()
    }

    fn copy_out(&self, source: &Self::Source, dst: &mut [u8]) {
        dst.copy_from_slice(&source[self.to_usize_range()])
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DirectRegionContent<'a> {
    pub content: &'a [u8],
}

impl<'a> RegionContent for DirectRegionContent<'a> {
    type Source = ();

    fn len(&self) -> usize {
        self.content.len()
    }

    fn copy_out(&self, _source: &Self::Source, dst: &mut [u8]) {
        dst.copy_from_slice(self.content)
    }
}

impl<U: RegionContent, const N: usize> Payload<usize, U, N> {
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn copy_data_out(&self, region_content_source: &U::Source) {
        for region in self.data.iter() {
            let dst = unsafe {
                slice::from_raw_parts_mut(
                    ptr::from_exposed_addr_mut(region.phys_addr_range.start.try_into().unwrap()),
                    (region.phys_addr_range.end - region.phys_addr_range.start)
                        .try_into()
                        .unwrap(),
                )
            };
            match &region.content {
                Some(src) => {
                    src.copy_out(region_content_source, dst);
                }
                None => {
                    // NOTE slice::fill is too slow
                    unsafe {
                        ptr::write_bytes(dst.as_mut_ptr(), 0, dst.len());
                    }
                }
            }
        }
    }
}

impl<U, const N: usize> Payload<usize, U, N> {
    pub fn sanity_check<T: PrimInt>(
        &self,
        platform_info: &PlatformInfo<T>,
        own_footprint: Range<usize>,
    ) {
        let memory = &platform_info.memory;
        assert!(any_range_contains(memory.iter(), &own_footprint));
        for region in self.data.iter() {
            assert!(any_range_contains(memory.iter(), &region.phys_addr_range));
            assert!(ranges_are_disjoint(&own_footprint, &region.phys_addr_range));
        }
    }
}

fn ranges_are_disjoint(this: &Range<usize>, that: &Range<usize>) -> bool {
    this.end.min(that.end) <= this.start.max(that.start)
}

fn range_contains<T: PrimInt>(this: &Range<T>, that: &Range<usize>) -> bool {
    this.start.to_usize().unwrap() <= that.start && that.end <= this.end.to_usize().unwrap()
}

fn any_range_contains<'a, T: PrimInt + 'a>(
    mut these: impl Iterator<Item = &'a Range<T>>,
    that: &Range<usize>,
) -> bool {
    these.any(|this| range_contains(this, that))
}
