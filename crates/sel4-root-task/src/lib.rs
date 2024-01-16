//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(cfg_target_thread_local)]
#![feature(never_type)]

use core::ffi::c_char;
use core::fmt;

pub use sel4_panicking_env::{abort, debug_print, debug_println};
pub use sel4_root_task_macros::root_task;

#[doc(inline)]
pub use sel4_panicking as panicking;

mod heap;
mod termination;

pub use heap::set_global_allocator_mutex_notification;
pub use termination::{Never, Termination};

#[cfg(target_thread_local)]
#[no_mangle]
unsafe extern "C" fn sel4_runtime_rust_entry(bootinfo: *const sel4::sys::seL4_BootInfo) -> ! {
    unsafe extern "C" fn cont_fn(cont_arg: *mut sel4_runtime_common::ContArg) -> ! {
        inner_entry(cont_arg.cast_const().cast())
    }

    sel4_runtime_common::initialize_tls_on_stack_and_continue(cont_fn, bootinfo.cast_mut().cast())
}

#[cfg(not(target_thread_local))]
#[no_mangle]
unsafe extern "C" fn sel4_runtime_rust_entry(bootinfo: *const sel4::sys::seL4_BootInfo) -> ! {
    inner_entry(bootinfo)
}

fn inner_entry(bootinfo: *const sel4::sys::seL4_BootInfo) -> ! {
    #[cfg(all(feature = "unwinding", panic = "unwind"))]
    {
        sel4_runtime_common::set_eh_frame_finder().unwrap();
    }

    unsafe {
        let bootinfo = sel4::BootInfo::from_ptr(bootinfo);
        sel4::set_ipc_buffer(bootinfo.ipc_buffer());
        __sel4_root_task__main(&bootinfo);
    }

    abort!("__sel4_root_task__main returned")
}

extern "Rust" {
    fn __sel4_root_task__main(bootinfo: &sel4::BootInfo);
}

#[doc(hidden)]
#[macro_export]
macro_rules! declare_main {
    ($main:expr) => {
        #[no_mangle]
        fn __sel4_root_task__main(bootinfo: &$crate::_private::BootInfo) {
            $crate::_private::run_main($main, bootinfo);
        }
    };
}

#[doc(hidden)]
#[allow(clippy::missing_safety_doc)]
pub fn run_main<T>(f: impl Fn(&sel4::BootInfo) -> T, bootinfo: &sel4::BootInfo)
where
    T: Termination,
    T::Error: fmt::Debug,
{
    let result = panicking::catch_unwind(|| f(bootinfo).report());
    match result {
        Ok(err) => abort!("main thread terminated with error: {err:?}"),
        Err(_) => abort!("uncaught panic in main thread"),
    }
}

fn debug_put_char(c: u8) {
    sel4::debug_put_char(c as c_char)
}

sel4_panicking_env::register_debug_put_char!(debug_put_char);

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

#[doc(hidden)]
pub const DEFAULT_STACK_SIZE: usize = 0x10000;

// For macros
#[doc(hidden)]
pub mod _private {
    pub use sel4::BootInfo;
    pub use sel4_runtime_common::declare_stack;

    pub use crate::heap::_private as heap;

    pub use crate::{declare_heap, declare_main, declare_root_task, run_main, DEFAULT_STACK_SIZE};
}
