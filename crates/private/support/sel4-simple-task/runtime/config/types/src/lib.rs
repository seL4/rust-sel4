//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

extern crate alloc;

use core::ops::Range;

use alloc::string::String;
use alloc::vec::Vec;

use rkyv::Archive;
use rkyv::rancor;
use rkyv::util::AlignedVec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(target_pointer_width = "32")]
type NativeWord = u32;

#[cfg(target_pointer_width = "64")]
type NativeWord = u64;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
#[rkyv(derive(Debug))]
pub struct Word(pub u64);

impl ArchivedWord {
    #[allow(clippy::useless_conversion)]
    pub fn to_native(&self) -> NativeWord {
        self.0.to_native().try_into().unwrap()
    }
}

pub type Address = u64;
pub type CPtrBits = Word;

pub type RuntimeConfig = GenericRuntimeConfig<Vec<u8>>;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct GenericRuntimeConfig<T> {
    pub static_heap: Option<Range<Address>>,
    pub static_heap_mutex_notification: Option<CPtrBits>,
    pub idle_notification: Option<CPtrBits>,
    pub threads: Vec<RuntimeThreadConfig>,
    pub image_identifier: Option<String>,
    pub app_config: T,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct RuntimeThreadConfig {
    pub ipc_buffer_addr: Address,
    pub endpoint: Option<CPtrBits>,
    pub reply_authority: Option<CPtrBits>,
}

impl<T> GenericRuntimeConfig<T> {
    pub fn traverse<U, V>(
        self,
        f: impl FnOnce(T) -> Result<U, V>,
    ) -> Result<GenericRuntimeConfig<U>, V> {
        Ok(GenericRuntimeConfig {
            static_heap: self.static_heap,
            static_heap_mutex_notification: self.static_heap_mutex_notification,
            idle_notification: self.idle_notification,
            threads: self.threads,
            image_identifier: self.image_identifier,
            app_config: f(self.app_config)?,
        })
    }
}

impl RuntimeConfig {
    pub fn to_bytes(&self) -> Result<AlignedVec, rancor::Error> {
        rkyv::to_bytes(self)
    }

    pub fn access(buf: &[u8]) -> Result<&<Self as Archive>::Archived, rancor::Error> {
        rkyv::access(buf)
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn access_unchecked(buf: &[u8]) -> &<Self as Archive>::Archived {
        unsafe { rkyv::access_unchecked(buf) }
    }
}
