//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

#[cfg(feature = "owned")]
extern crate alloc;

use core::ops::Range;

#[cfg(feature = "owned")]
use alloc::vec::Vec;

#[cfg(feature = "owned")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct PlatformInfo<'a, T> {
    pub memory: &'a [Range<T>],
    pub devices: &'a [Range<T>],
}

#[cfg(feature = "owned")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedPlatformInfo {
    pub memory: Vec<Range<u64>>,
    pub devices: Vec<Range<u64>>,
}
