#![no_std]
#![feature(associated_type_bounds)]

use core::borrow::Borrow;
use core::ops::Range;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Payload<T> {
    pub info: PayloadInfo,
    pub data: T,
}

pub trait Regions {
    type RegionContent: Borrow<[u8]>;
    type RegionsIterItem: Borrow<Region<Self::RegionContent>>;
    type RegionsIter: Iterator<Item = Self::RegionsIterItem>;

    fn iter_regions(&self) -> Self::RegionsIter;
}

impl<'a, T: Clone + IntoIterator<Item = &'a Region<&'a [u8]>>> Regions for T {
    type RegionContent = &'a [u8];
    type RegionsIterItem = &'a Region<Self::RegionContent>;
    type RegionsIter = T::IntoIter;

    fn iter_regions(&self) -> Self::RegionsIter {
        self.clone().into_iter()
    }
}

impl<T: Regions> Payload<T> {
    pub fn data(&self) -> T::RegionsIter {
        self.data.iter_regions()
    }
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
    pub phys_to_virt_offset: i64,
    pub virt_entry: u64,
}

impl ImageInfo {
    pub fn virt_addr_range(&self) -> Range<u64> {
        self.phys_to_virt(self.phys_addr_range.start)..self.phys_to_virt(self.phys_addr_range.end)
    }

    pub fn phys_to_virt(&self, paddr: u64) -> u64 {
        u64::try_from(i64::try_from(paddr).unwrap() + self.phys_to_virt_offset).unwrap()
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
