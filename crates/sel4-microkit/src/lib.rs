//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

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

mod entry;
mod heap;
mod printing;

pub mod panicking;

#[sel4::sel4_cfg(PRINTING)]
pub use printing::{debug_print, debug_println};

/// Declares a function to be the the protection domain's initialization function.
///
/// For example:
///
/// ```rust
/// #[protection_domain]
/// fn init() -> impl Handler {
///     todo!()
/// }
/// ```
///
/// The initialization function have a signature of the form:
///
/// ```rust
/// fn<T: Handler>() -> T
/// ```
///
/// (See [`Handler`])
///
/// This macro an optional `heap_size` parameter, whose value can be any expression of type `usize`:
///
/// ```rust
/// #[protection_domain(heap_size = <heap_size_expr: usize>)]
/// ```
///
/// If this parameter is provided, the macro creates a `#[global_allocator]`, backed by a static
/// heap of the specified size. If this parameter is not specified, no `#[global_allocator]` will be
/// automatically declared, and, unless one is manually declared, heap allocations will result in a
/// link-time error.
pub use sel4_microkit_macros::protection_domain;

#[doc(hidden)]
#[macro_export]
macro_rules! declare_protection_domain {
    {
        init = $init:expr $(,)?
    } => {
        $crate::_private::declare_init!($init);
    };
    {
        init = $init:expr,
        heap_size = $heap_size:expr $(,)?
    } => {
        $crate::_private::declare_init!($init);
        $crate::_private::declare_heap!($heap_size);
    };
}

// For macros
#[doc(hidden)]
pub mod _private {
    pub use crate::heap::_private as heap;

    pub use crate::{declare_heap, declare_init, declare_protection_domain, entry::run_main};
}
