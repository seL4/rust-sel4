#![no_std]

use core::ops::Range;

use heapless::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub const MAX_NUM_REGIONS: usize = 16;

pub type ConcretePayload = Payload<IndirectRegionContent, MAX_NUM_REGIONS>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Payload<T, const N: usize> {
    pub info: PayloadInfo,
    pub data: Vec<Region<T>, N>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PayloadInfo {
    pub kernel_image: ImageInfo,
    pub user_image: ImageInfo,
    pub fdt_phys_addr_range: Option<Range<u64>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ImageInfo {
    pub phys_addr_range: Range<u64>,
    pub phys_to_virt_offset: i128,
    pub virt_entry: u64,
}

impl ImageInfo {
    pub fn virt_addr_range(&self) -> Range<u64> {
        self.phys_to_virt(self.phys_addr_range.start)..self.phys_to_virt(self.phys_addr_range.end)
    }

    pub fn phys_to_virt(&self, paddr: u64) -> u64 {
        u64::try_from(i128::from(paddr) + self.phys_to_virt_offset).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Region<T> {
    pub phys_addr_range: Range<u64>,
    pub content: Option<T>,
}

impl<T> Region<T> {
    pub fn traverse<U, E>(&self, mut f: impl FnMut(&T) -> Result<U, E>) -> Result<Region<U>, E> {
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
pub struct IndirectRegionContent {
    pub content_range: Range<usize>,
}

impl RegionContent for IndirectRegionContent {
    type Source = [u8];

    fn len(&self) -> usize {
        self.content_range.end - self.content_range.start
    }

    fn copy_out(&self, source: &Self::Source, dst: &mut [u8]) {
        dst.copy_from_slice(&source[self.content_range.clone()])
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
