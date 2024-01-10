//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        mod with_alloc;
        pub use with_alloc::Backtrace;
    }
}

#[cfg(feature = "postcard")]
mod with_postcard;

#[cfg(feature = "symbolize")]
mod with_symbolize;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Preamble<T> {
    pub image: T,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Postamble {
    pub error: Option<Error>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Entry {
    pub stack_frame: StackFrame,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct StackFrame {
    pub ip: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Error {
    pub unwind_reason_code: i32,
}
