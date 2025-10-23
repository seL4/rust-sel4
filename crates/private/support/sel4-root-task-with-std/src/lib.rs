//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![feature(cfg_target_thread_local)]
#![feature(linkage)]
#![feature(never_type)]

use sel4::sel4_cfg_if;

pub use sel4_panicking_env::{abort, debug_print, debug_println};

mod entry;
mod termination;

pub use termination::{Never, Termination};

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
}

pub const DEFAULT_STACK_SIZE: usize = 1024
    * if cfg!(panic = "unwind") && cfg!(debug_assertions) {
        128
    } else {
        64
    };

sel4_cfg_if! {
    if #[sel4_cfg(PRINTING)] {
        use sel4::debug_put_char;
    } else {
        fn debug_put_char(_: u8) {}
    }
}

sel4_panicking_env::register_debug_put_char!(
    #[linkage = "weak"]
    debug_put_char
);

// For macros
#[doc(hidden)]
pub mod _private {
    pub use sel4::BootInfoPtr;
    pub use sel4_runtime_common::declare_stack;

    pub use crate::{DEFAULT_STACK_SIZE, declare_main, declare_root_task, entry::run_main};
}
