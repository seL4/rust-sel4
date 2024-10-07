//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

//! A runtime for root tasks.
//!
//! This crate defines an entrypoint at `_start` that performs the following Rust language runtime
//! and [`libsel4`](sel4) initialization:
//! - Set up the stack
//! - Initialize thread-local storage (using stack memory)
//! - Set up exception handling
//! - Set the seL4 IPC buffer for `libsel4` (using [`sel4::set_ipc_buffer`])
//! - Run C-style constructors (from `__init_array_start`/`__init_array_end`)
//!
//! After initialization, the entrypoint calls the root task main function, which must be declared
//! with [`#[root_task]`](root_task). For example:
//!
//! ```rust
//! #[root_task]
//! fn main(bootinfo: &sel4::BootInfo) -> ! {
//!     todo!()
//! }
//! ```
//!
//! Building root tasks using this crate does not require a custom linker script when using `LLD`.
//! In particular, this crate is tested with the `LLD` binary that ships with `rustc` (`rust-lld`).
//! Using a GNU linker will likely require a custom linker script.

#![no_std]
#![feature(cfg_target_thread_local)]
#![feature(linkage)]
#![feature(never_type)]

pub use sel4_panicking_env::abort;

#[doc(inline)]
pub use sel4_panicking as panicking;

mod entry;
mod heap;
mod printing;
mod termination;

pub use heap::set_global_allocator_mutex_notification;
pub use termination::{Never, Termination};

#[sel4::sel4_cfg(PRINTING)]
pub use sel4_panicking_env::{debug_print, debug_println};

/// Declares a function to be the root task's main function.
///
/// For example:
///
/// ```rust
/// #[root_task]
/// fn main(bootinfo: &sel4::BootInfo) -> ! {
///     todo!()
/// }
/// ```
///
/// The main function have a signature of the form:
///
/// ```rust
/// fn<T: Termination>(&sel4::BootInfoPtr) -> T
/// ```
///
/// (See [`Termination`])
///
/// This macro takes two optional parameters, whose values can be any expression of type `usize`:
///
/// ```rust
/// #[root_task(
///     stack_size = <stack_size_expr: usize>,
///     heap_size = <heap_size_expr: usize>,
/// )]
/// ```
///
/// - `stack_size`: Declares the size of the initial thread's stack, in bytes. Note that this
///   includes space for thread-local storage. If absent, [`DEFAULT_STACK_SIZE`] will be used.
/// - `heap_size`: Creates a `#[global_allocator]`, backed by a static heap of the specified size.
///   If this parameter is not specified, no `#[global_allocator]` will be automatically declared,
///   and, unless one is manually declared, heap allocations will result in a link-time error.
///
/// Note that, if both parameters are provided, they must appear in the order above.
pub use sel4_root_task_macros::root_task;

#[doc(hidden)]
#[macro_export]
macro_rules! declare_root_task {
    {
        main = $main:expr $(,)?
    } => {
        $crate::_private::declare_root_task! {
            main = $main,
            stack_size = $crate::_private::DEFAULT_STACK_SIZE,
        }
    };
    {
        main = $main:expr,
        stack_size = $stack_size:expr $(,)?
    } => {
        $crate::_private::declare_main!($main);
        $crate::_private::declare_stack!($stack_size);
    };
    {
        main = $main:expr,
        $(stack_size = $stack_size:expr,)?
        heap_size = $heap_size:expr $(,)?
    } => {
        $crate::_private::declare_heap!($heap_size);
        $crate::_private::declare_root_task! {
            main = $main,
            $(stack_size = $stack_size,)?
        }
    };
}

/// The default stack size used by [`#[root_task]`](crate::root_task).
pub const DEFAULT_STACK_SIZE: usize = 1024
    * if cfg!(panic = "unwind") && cfg!(debug_assertions) {
        128
    } else {
        64
    };

// For macros
#[doc(hidden)]
pub mod _private {
    pub use sel4::BootInfoPtr;
    pub use sel4_runtime_common::declare_stack;

    pub use crate::heap::_private as heap;

    pub use crate::{
        declare_heap, declare_main, declare_root_task, entry::run_main, DEFAULT_STACK_SIZE,
    };
}
