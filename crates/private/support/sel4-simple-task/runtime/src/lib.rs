//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(never_type)]
#![feature(thread_local)]
#![allow(internal_features)]
#![allow(clippy::useless_conversion)]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::panic::PanicInfo;
use core::slice;

use sel4_dlmalloc::StaticHeapBounds;
use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;
use sel4_panicking_env::{abort, AbortInfo};
use sel4_simple_task_runtime_config_types::RuntimeConfig;
use sel4_simple_task_threading::StaticThread;

#[cfg(not(target_arch = "arm"))]
use sel4_backtrace_simple::SimpleBacktracing;

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
    sel4_runtime_common::maybe_with_tls(|| cont_fn(config, thread_index))
}

#[allow(clippy::missing_safety_doc)]
pub fn cont_fn(config: RuntimeConfig<'static>, thread_index: usize) -> ! {
    let thread_config = &config.threads()[thread_index];

    THREAD_INDEX.set(thread_index).unwrap();

    sel4::set_ipc_buffer(unsafe {
        (usize::try_from(thread_config.ipc_buffer_addr()).unwrap() as *mut sel4::IpcBuffer)
            .as_mut()
            .unwrap()
    });

    if thread_index == 0 {
        CONFIG.set(config.clone()).unwrap();

        sel4_runtime_common::maybe_set_eh_frame_finder().unwrap();
        sel4_ctors_dtors::run_ctors().unwrap();

        #[cfg(feature = "alloc")]
        {
            global_allocator::init(
                get_static_heap_mutex_notification(),
                get_static_heap_bounds(),
            );
        }

        sel4_panicking::set_hook(&panic_hook);

        unsafe {
            __sel4_simple_task_main(config.arg());
        }
    } else {
        let endpoint =
            sel4::cap::Endpoint::from_bits(thread_config.endpoint().unwrap().try_into().unwrap());
        let reply_authority = {
            sel4::sel4_cfg_if! {
                if #[sel4_cfg(KERNEL_MCS)] {
                    sel4::cap::Reply::from_bits(thread_config.reply_authority().unwrap().try_into().unwrap())
                } else {
                    assert!(thread_config.reply_authority().is_none());
                    sel4::ImplicitReplyAuthority::default()
                }
            }
        };
        unsafe {
            StaticThread::recv_and_run(endpoint, reply_authority);
        }
    }

    idle()
}

pub fn try_idle() {
    CONFIG
        .get()
        .and_then(RuntimeConfig::idle_notification)
        .map(sel4::CPtrBits::try_from)
        .map(Result::unwrap)
        .map(sel4::cap::Notification::from_bits)
        .map(sel4::cap::Notification::wait);
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

sel4_panicking_env::register_debug_put_char!(sel4::debug_put_char);

fn panic_hook(info: &PanicInfo<'_>) {
    debug_println!("{}", info);

    #[cfg(not(target_arch = "arm"))]
    {
        get_backtracing().collect_and_send();
    }
}

fn get_static_heap_bounds() -> StaticHeapBounds {
    let addrs = CONFIG.get().unwrap().static_heap().unwrap();
    StaticHeapBounds::new(
        usize::try_from(addrs.start).unwrap() as *mut _,
        (addrs.end - addrs.start).try_into().unwrap(),
    )
}

fn get_static_heap_mutex_notification() -> sel4::cap::Notification {
    CONFIG
        .get()
        .unwrap()
        .static_heap_mutex_notification()
        .map(sel4::CPtrBits::try_from)
        .map(Result::unwrap)
        .map(sel4::cap::Notification::from_bits)
        .unwrap()
}

#[cfg(not(target_arch = "arm"))]
pub fn get_backtracing() -> SimpleBacktracing {
    SimpleBacktracing::new(get_backtrace_image_identifier())
}

#[allow(dead_code)]
fn get_backtrace_image_identifier() -> Option<&'static str> {
    CONFIG.get().unwrap().image_identifier()
}

// // //

// For macros
#[doc(hidden)]
pub mod _private {
    pub use crate::declare_main::_private as declare_main;
}
