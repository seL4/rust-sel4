//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(never_type)]
#![feature(thread_local)]
#![allow(internal_features)]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::ffi::c_char;
use core::ptr;
use core::slice;

use sel4::Endpoint;
use sel4_backtrace_simple::SimpleBacktracing;
use sel4_dlmalloc::StaticHeapBounds;
use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;
use sel4_panicking::ExternalPanicInfo;
use sel4_panicking_env::{abort, AbortInfo};
use sel4_simple_task_runtime_config_types::RuntimeConfig;
use sel4_simple_task_threading::StaticThread;

mod declare_main;
mod termination;

#[cfg(feature = "alloc")]
mod global_allocator;

pub use sel4_panicking_env::{debug_print, debug_println};
pub use sel4_simple_task_runtime_macros::{main, main_json, main_postcard};

extern "Rust" {
    pub(crate) fn __sel4_simple_task_main(arg: &[u8]);
}

static CONFIG: ImmediateSyncOnceCell<RuntimeConfig<'static>> = ImmediateSyncOnceCell::new();

#[thread_local]
static THREAD_INDEX: ImmediateSyncOnceCell<usize> = ImmediateSyncOnceCell::new();

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _start(config: *const u8, config_size: usize, thread_index: usize) -> ! {
    let config = RuntimeConfig::new(slice::from_raw_parts(config, config_size));
    let mut cont_arg = ContArg {
        config,
        thread_index,
    };
    sel4_runtime_common::initialize_tls_on_stack_and_continue(
        cont_fn,
        ptr::addr_of_mut!(cont_arg).cast(),
    )
}

pub struct ContArg {
    config: RuntimeConfig<'static>,
    thread_index: usize,
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn cont_fn(cont_arg: *mut sel4_runtime_common::ContArg) -> ! {
    let cont_arg: &ContArg = &*cont_arg.cast_const().cast();

    let config = &cont_arg.config;
    let thread_index = cont_arg.thread_index;
    let thread_config = &config.threads()[thread_index];

    THREAD_INDEX.set(thread_index).unwrap();

    sel4::set_ipc_buffer(sel4::IPCBuffer::from_ptr(
        usize::try_from(thread_config.ipc_buffer_addr()).unwrap() as *mut _,
    ));

    if thread_index == 0 {
        CONFIG.set(config.clone()).unwrap();
        sel4_runtime_common::set_eh_frame_finder().unwrap();
        sel4_panicking::set_hook(&panic_hook);
        __sel4_simple_task_main(config.arg());
    } else {
        let endpoint = Endpoint::from_bits(thread_config.endpoint().unwrap());
        let reply_authority = {
            sel4::sel4_cfg_if! {
                if #[cfg(KERNEL_MCS)] {
                    sel4::Reply::from_bits(thread_config.reply_authority().unwrap())
                } else {
                    assert!(thread_config.reply_authority().is_none());
                    sel4::ImplicitReplyAuthority
                }
            }
        };
        StaticThread::recv_and_run(endpoint, reply_authority);
    }

    idle()
}

pub fn try_idle() {
    CONFIG
        .get()
        .and_then(RuntimeConfig::idle_notification)
        .map(sel4::Notification::from_bits)
        .map(sel4::Notification::wait);
}

pub fn idle() -> ! {
    try_idle();
    abort!("idle failed")
}

fn abort_hook(info: Option<&AbortInfo>) {
    match info {
        Some(info) => debug_println!("{}", info),
        None => debug_println!("(aborted)"),
    }
    try_idle()
}

sel4_panicking_env::register_abort_hook!(abort_hook);

fn debug_put_char(c: u8) {
    sel4::debug_put_char(c as c_char)
}

sel4_panicking_env::register_debug_put_char!(debug_put_char);

fn panic_hook(info: &ExternalPanicInfo<'_>) {
    debug_println!("{}", info);
    get_backtracing().collect_and_send();
}

fn get_static_heap_bounds() -> StaticHeapBounds {
    let addrs = CONFIG.get().unwrap().static_heap().unwrap();
    StaticHeapBounds::new(
        usize::try_from(addrs.start).unwrap() as *mut _,
        (addrs.end - addrs.start).try_into().unwrap(),
    )
}

fn get_static_heap_mutex_notification() -> sel4::Notification {
    CONFIG
        .get()
        .unwrap()
        .static_heap_mutex_notification()
        .map(sel4::Notification::from_bits)
        .unwrap()
}

pub fn get_backtracing() -> SimpleBacktracing {
    SimpleBacktracing::new(get_backtrace_image_identifier())
}

fn get_backtrace_image_identifier() -> Option<&'static str> {
    CONFIG.get().unwrap().image_identifier()
}

// // //

// For macros
#[doc(hidden)]
pub mod _private {
    pub use crate::declare_main::_private as declare_main;
}
