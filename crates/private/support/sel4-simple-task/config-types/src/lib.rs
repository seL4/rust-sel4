//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::marker::PhantomData;

use serde::{Deserialize, Serialize};

cfg_if::cfg_if! {
    if #[cfg(target_env = "sel4")] {
        mod when_sel4;
        pub use when_sel4::*;
    } else {
        mod when_not_sel4;
        pub use when_not_sel4::*;
    }
}

pub type RawConfigWord = u64;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConfigBadge(RawConfigWord);

impl ConfigBadge {
    pub fn new(word: RawConfigWord) -> Self {
        Self(word)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConfigCPtrBits(u64);

impl ConfigCPtrBits {
    pub fn new(word: RawConfigWord) -> Self {
        Self(word)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConfigCPtr<T> {
    phantom: PhantomData<T>,
    bits: ConfigCPtrBits,
}

impl<T> ConfigCPtr<T> {
    pub fn new(bits: ConfigCPtrBits) -> Self {
        Self {
            phantom: PhantomData,
            bits,
        }
    }
}
