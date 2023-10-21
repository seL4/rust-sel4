//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(int_roundings)]
#![feature(result_flattening)]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::ops::Range;
use core::str;

use zerocopy::{AsBytes, FromBytes, FromZeroes, Ref};

mod zerocopy_helpers;

pub use zerocopy_helpers::{
    InvalidZerocopyOptionTag, NativeWord, ZerocopyOptionWord, ZerocopyOptionWordRange,
    ZerocopyWord, ZerocopyWordRange,
};

#[cfg(feature = "alloc")]
mod with_alloc;

#[cfg(feature = "alloc")]
pub use with_alloc::{RuntimeConfigForPacking, RuntimeThreadConfigForPacking};

pub type Address = NativeWord;
pub type CPtrBits = NativeWord;

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, AsBytes, FromBytes, FromZeroes)]
struct Head {
    static_heap: ZerocopyOptionWordRange,
    static_heap_mutex_notification: ZerocopyOptionWord,
    idle_notification: ZerocopyOptionWord,
    threads: ZerocopyWordRange,
    image_identifier: ZerocopyOptionWordRange,
    arg: ZerocopyWordRange,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, AsBytes, FromBytes, FromZeroes)]
struct Thread {
    ipc_buffer_addr: ZerocopyWord,
    endpoint: ZerocopyOptionWord,
    reply_authority: ZerocopyOptionWord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfig<'a> {
    bytes: &'a [u8],
}

impl<'a> RuntimeConfig<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub fn static_heap(&self) -> Option<Range<Address>> {
        self.head().static_heap.try_into_native().unwrap()
    }

    pub fn static_heap_mutex_notification(&self) -> Option<CPtrBits> {
        self.head()
            .static_heap_mutex_notification
            .try_into_native()
            .unwrap()
    }

    pub fn idle_notification(&self) -> Option<CPtrBits> {
        self.head().idle_notification.try_into_native().unwrap()
    }

    pub fn threads(&self) -> &[RuntimeThreadConfig] {
        Ref::new_slice(self.index(self.head().threads.try_into_native().unwrap()))
            .unwrap()
            .into_slice()
    }

    pub fn image_identifier(&self) -> Option<&str> {
        self.head()
            .image_identifier
            .try_into_native()
            .unwrap()
            .map(|range| str::from_utf8(self.index(range)).unwrap())
    }

    pub fn arg(&self) -> &[u8] {
        self.index(self.head().arg.try_into_native().unwrap())
    }

    fn head(&self) -> &Head {
        let (head, _) = Ref::<_, Head>::new_from_prefix(self.bytes).unwrap();
        head.into_ref()
    }

    fn index(&self, range: Range<usize>) -> &[u8] {
        &self.bytes[range]
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, AsBytes, FromBytes, FromZeroes)]
pub struct RuntimeThreadConfig {
    inner: Thread,
}

impl RuntimeThreadConfig {
    pub fn ipc_buffer_addr(&self) -> Address {
        self.inner.ipc_buffer_addr.get()
    }

    pub fn endpoint(&self) -> Option<CPtrBits> {
        self.inner.endpoint.try_into_native().unwrap()
    }

    pub fn reply_authority(&self) -> Option<CPtrBits> {
        self.inner.reply_authority.try_into_native().unwrap()
    }
}
