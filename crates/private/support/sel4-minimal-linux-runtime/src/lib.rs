//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(never_type)]

use core::panic::UnwindSafe;

pub use sel4_minimal_linux_syscalls::exit;
pub use sel4_panicking::catch_unwind;
pub use sel4_panicking_env::{abort, debug_println};

pub use sel4_minimal_linux_runtime_macros::main;

#[doc(hidden)]
#[macro_export]
macro_rules! declare_main {
    {
        main = $main:expr $(,)?
    } => {
        $crate::_private::declare_main! {
            main = $main,
            stack_size = $crate::_private::DEFAULT_STACK_SIZE,
        }
    };
    {
        main = $main:expr,
        stack_size = $stack_size:expr $(,)?
    } => {
        $crate::_private::declare_stack!($stack_size);

        const _: () = {
            #[allow(non_snake_case)]
            #[unsafe(no_mangle)]
            fn __sel4_minimal_linux_runtime__main() {
                $crate::_private::_run_main($main);
            }
        };
    };
    {
        main = $main:expr,
        $(stack_size = $stack_size:expr,)?
        heap_size = $heap_size:expr $(,)?
    } => {{
        const _: () = {
            static STATIC_HEAP: $crate::private_::StaticHeap<{ $heap_size }> =
                $crate::private_::StaticHeap::new();

            #[global_allocator]
            static GLOBAL_ALLOCATOR: $crate::private_::StaticDlmalloc<$crate::private_::RawOneShotMutex> =
                $crate::private_::StaticDlmalloc::new(STATIC_HEAP.bounds());
        };

        $crate::_private::declare_main! {
            main = $main,
            $(stack_size = $stack_size,)?
        }
    }};
}

pub const DEFAULT_STACK_SIZE: usize = 1024
    * if cfg!(panic = "unwind") && cfg!(debug_assertions) {
        128
    } else {
        64
    };

sel4_runtime_common::declare_entrypoint_with_stack_init! {
    entrypoint()
}

fn entrypoint() -> ! {
    unsafe {
        __sel4_minimal_linux_runtime__main();
    }
    abort!("main returned")
}

unsafe extern "Rust" {
    fn __sel4_minimal_linux_runtime__main();
}

#[doc(hidden)]
#[allow(clippy::missing_safety_doc)]
pub fn _run_main<F>(f: F) -> !
where
    F: FnOnce() + UnwindSafe,
{
    let r = catch_unwind(move || f());
    match r {
        Ok(()) => exit(0),
        Err(()) => abort!("uncaught panic in main"),
    }
}

// For macros
#[doc(hidden)]
pub mod _private {
    pub use crate::{_run_main, DEFAULT_STACK_SIZE, declare_main};
    pub use one_shot_mutex::sync::RawOneShotMutex;
    pub use sel4_dlmalloc::{StaticDlmalloc, StaticHeap};
    pub use sel4_runtime_common::declare_stack;
}
