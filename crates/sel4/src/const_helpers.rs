#![allow(clippy::assertions_on_constants)]

use sel4_config::sel4_cfg_attr;

pub(crate) const fn u32_into_usize(x: u32) -> usize {
    assert!(u32::BITS <= usize::BITS);
    x as usize
}

#[sel4_cfg_attr(not(KERNEL_MCS), allow(dead_code))]
pub(crate) const fn usize_max(x: usize, y: usize) -> usize {
    if x >= y {
        x
    } else {
        y
    }
}
