#![no_std]

use core::ops::Range;

#[derive(Debug, Clone)]
pub struct PlatformInfo<'a, T> {
    pub memory: &'a [Range<T>],
    pub devices: &'a [Range<T>],
}
