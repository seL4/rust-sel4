//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

//! Polyfill for the proposed `core::process::crash`.
//!
//! See <https://github.com/rust-lang/rust/pull/164731>.

#![no_std]
#![cfg_attr(before_crash, feature(core_intrinsics))]
#![cfg_attr(before_crash, allow(internal_features))]
#![cfg_attr(not(before_crash), feature(abort_immediate))]

#[inline]
pub fn crash() -> ! {
    cfg_select! {
        before_crash => core::intrinsics::abort(),
        _ => core::process::abort_immediate(),
    }
}
