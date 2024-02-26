//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(cfg_target_thread_local)]

//! A foundation for pure-Rust [seL4 Microkit](https://github.com/seL4/microkit) protection domains.
//!
//! See the [seL4 Microkit manual](https://github.com/seL4/microkit/blob/main/docs/manual.md) for
//! non-Rust-specific documentation about the seL4 Microkit.
//!
//! See [the demo](https://github.com/seL4/rust-microkit-demo) for a concrete example of
//! this crate in action.
//!
//! This crate depends, at build time, on the libsel4 headers. It requires that either
//! `$SEL4_INCLUDE_DIRS` contains a colon-separated list of include paths for the libsel4 headers,
//! or that `$SEL4_PREFIX` is set, in which case `$SEL4_PREFIX/libsel4/include` is used.
//!
//! The `microkit` tool expects protection domain binaries to expose a few symbols. All protection
//! domains must contain the symbol `__sel4_ipc_buffer_obj`. Furthermore, for protection domains
//! with memory regions, the `microkit` tool injects the addresses of these memory regions at build
//! time by patching designated symbols. The
//! [`*-sel4-microkit{,-minimal}.json`](https://github.com/seL4/rust-sel4/tree/main/support/targets)
//! `rustc` target specs distributed as part of the [rust-sel4
//! project](https://github.com/seL4/rust-sel4) provide `__sel4_ipc_buffer_obj`, and the
//! [`memory_region_symbol`] macro provides a conveneint way to declare memory region address
//! symbols.
//!
//! Use the [`protection_domain`] macro to declare the initialization function, stack size, and,
//! optionally, heap and heap size.

#[cfg(feature = "alloc")]
extern crate alloc;

pub use sel4_microkit_base::*;
pub use sel4_microkit_macros::protection_domain;

mod defer;
mod entry;
mod handler;
mod heap;
mod printing;

pub mod panicking;

pub use defer::{DeferredAction, DeferredActionInterface, DeferredActionSlot};
pub use handler::{Handler, Infallible, NullHandler};
pub use printing::{debug_print, debug_println};

/// Declares the initialization function, stack size, and, optionally, heap and heap size.
///
/// See the [`protection_domain`] attribute macro for more detail.
#[macro_export]
macro_rules! declare_protection_domain {
    {
        init = $init:expr $(,)?
    } => {
        $crate::_private::declare_protection_domain! {
            init = $init,
            stack_size = $crate::_private::DEFAULT_STACK_SIZE,
        }
    };
    {
        init = $init:expr,
        stack_size = $stack_size:expr $(,)?
    } => {
        $crate::_private::declare_init!($init);
        $crate::_private::declare_stack!($stack_size);
    };
    {
        init = $init:expr,
        $(stack_size = $stack_size:expr,)?
        heap_size = $heap_size:expr $(,)?
    } => {
        $crate::_private::declare_heap!($heap_size);
        $crate::_private::declare_protection_domain! {
            init = $init,
            $(stack_size = $stack_size,)?
        }
    };
}

#[doc(hidden)]
pub const DEFAULT_STACK_SIZE: usize = 0x10000;

// For macros
#[doc(hidden)]
pub mod _private {
    pub use sel4_runtime_common::declare_stack;

    pub use crate::heap::_private as heap;

    pub use crate::{
        declare_heap, declare_init, declare_protection_domain, entry::run_main, DEFAULT_STACK_SIZE,
    };
}
