#![no_std]

use core::ops::Range;

#[derive(Debug, Clone)]
pub struct PlatformInfo<'a> {
    pub memory: &'a [Range<u64>],
    pub devices: &'a [Range<u64>],
}
