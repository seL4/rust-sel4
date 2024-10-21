//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::marker::PhantomData;

use serde::{Deserialize, Serialize};

cfg_if::cfg_if! {
    if #[cfg(target_os = "none")] {
        mod when_sel4;
        pub use when_sel4::*;
    } else {
        mod when_not_sel4;
        pub use when_not_sel4::*;
    }
}

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
pub struct ConfigCPtrBits(RawConfigWord);

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
