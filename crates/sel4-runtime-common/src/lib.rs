//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(cfg_target_thread_local)]
#![feature(exclusive_wrapper)]
#![feature(stmt_expr_attributes)]

#[cfg(feature = "start")]
mod start;

#[cfg(feature = "static-heap")]
mod static_heap;

#[cfg(any(all(feature = "tls", target_thread_local), feature = "unwinding"))]
mod phdrs;

#[cfg(any(all(feature = "tls", target_thread_local), feature = "unwinding"))]
pub use phdrs::*;

#[doc(hidden)]
pub mod _private {
    #[cfg(feature = "start")]
    pub use crate::start::_private as start;
    #[cfg(feature = "static-heap")]
    pub use crate::static_heap::_private as static_heap;
}
