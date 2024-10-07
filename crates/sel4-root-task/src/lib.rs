//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(cfg_target_thread_local)]
#![feature(linkage)]
#![feature(never_type)]

pub use sel4_panicking_env::abort;
pub use sel4_root_task_macros::root_task;

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
