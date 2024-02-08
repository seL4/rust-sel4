//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

#![allow(clippy::assertions_on_constants)]

use sel4_config::sel4_cfg_attr;

use crate::Word;

const _: () = assert!(Word::BITS == usize::BITS);
const _: () = assert!(u32::BITS <= usize::BITS);

pub(crate) const fn word_into_usize(x: Word) -> usize {
    x as usize
}

pub(crate) const fn usize_into_word(x: usize) -> Word {
    x as Word
}

pub(crate) const fn u32_into_usize(x: u32) -> usize {
    x as usize
}

#[allow(dead_code)]
pub(crate) const fn u32_into_word(x: u32) -> Word {
    x as Word
}

#[sel4_cfg_attr(not(KERNEL_MCS), allow(dead_code))]
pub(crate) const fn usize_max(x: usize, y: usize) -> usize {
    if x >= y {
        x
    } else {
        y
    }
}
