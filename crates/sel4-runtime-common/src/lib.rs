//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(cfg_target_thread_local)]

mod ctors;

pub use ctors::run_ctors;

#[cfg(feature = "start")]
mod start;

#[cfg(any(
    all(feature = "tls", target_thread_local),
    all(feature = "unwinding", panic = "unwind")
))]
mod phdrs;

#[cfg(any(
    all(feature = "tls", target_thread_local),
    all(feature = "unwinding", panic = "unwind")
))]
pub use phdrs::*;

#[doc(hidden)]
pub mod _private {
    #[cfg(feature = "start")]
    pub use crate::start::_private as start;
}

#[cfg(target_arch = "arm")]
core::arch::global_asm! {
    r#"
        .global __aeabi_read_tp

        .section .text

        __aeabi_read_tp:
            mrc p15, 0, r0, c13, c0, 2
            bx lr
    "#
}
