//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

//! Polyfill for the never type, which was stabilized in Rust 1.100.
//!
//! Using `!` through [`Never`] does not require `#![feature(never_type)]` with older toolchains.

#![no_std]
#![cfg_attr(before_never_type, feature(never_type))]

pub type Never = !;
