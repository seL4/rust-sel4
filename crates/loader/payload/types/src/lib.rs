#![no_std]

use core::ops::Range;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Payload<'a> {
    pub info: PayloadInfo,
    pub data: &'a [Region<&'a [u8]>],
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
