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

use rkyv::Archive;

use sel4_dlmalloc::StaticHeapBounds;
use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;
use sel4_panicking_env::{AbortInfo, abort};
use sel4_simple_task_runtime_config_types::{RuntimeConfig, RuntimeThreadConfig};
use sel4_simple_task_threading::StaticThread;

#[cfg(not(target_arch = "arm"))]
use sel4_backtrace_simple::SimpleBacktracing;

mod declare_main;
mod termination;

#[cfg(feature = "alloc")]
mod global_allocator;

pub use sel4_panicking_env::{debug_print, debug_println};
pub use sel4_simple_task_runtime_macros::{main, main_json, main_postcard};

unsafe extern "Rust" {
    pub(crate) fn __sel4_simple_task_main(arg: &[u8]);
}

static CONFIG: ImmediateSyncOnceCell<&'static <RuntimeConfig as Archive>::Archived> =
    ImmediateSyncOnceCell::new();

#[thread_local]
static THREAD_INDEX: ImmediateSyncOnceCell<usize> = ImmediateSyncOnceCell::new();

sel4_runtime_common::declare_entrypoint!();

sel4_runtime_common::declare_rust_entrypoint! {
    entrypoint(config: *const u8, config_size: usize, thread_index: usize)
    global_init if thread_index == 0
}

#[allow(clippy::missing_safety_doc)]
unsafe fn entrypoint(config: *const u8, config_size: usize, thread_index: usize) -> ! {
    let config =
        unsafe { RuntimeConfig::access_unchecked(slice::from_raw_parts(config, config_size)) };

    let thread_config = &config.threads[thread_index];

    sel4::set_ipc_buffer(unsafe {
        (usize::try_from(thread_config.ipc_buffer_addr).unwrap() as *mut sel4::IpcBuffer)
            .as_mut()
            .unwrap()
    });

    THREAD_INDEX.set(thread_index).unwrap_or_else(|_| abort!());

    if thread_index == 0 {
        CONFIG.set(config).unwrap_or_else(|_| abort!());

        #[cfg(feature = "alloc")]
        {
            global_allocator::init(
                get_static_heap_mutex_notification(),
                get_static_heap_bounds(),
            );
        }

        sel4_panicking::set_hook(&panic_hook);

        unsafe {
            __sel4_simple_task_main(config.app_config.as_slice());
        }
    } else {
        run_secondary_thread(thread_config)
    }

    idle()
}

// TODO assert global_init_complete
fn run_secondary_thread(thread_config: &<RuntimeThreadConfig as Archive>::Archived) {
    let endpoint =
        sel4::cap::Endpoint::from_bits(thread_config.endpoint.as_ref().unwrap().to_native());
    let reply_authority = {
        sel4::sel4_cfg_if! {
            if #[sel4_cfg(KERNEL_MCS)] {
                sel4::cap::Reply::from_bits(thread_config.reply_authority().unwrap().to_native().try_into().unwrap())
            } else {
                assert!(thread_config.reply_authority.is_none());
                sel4::ImplicitReplyAuthority::default()
            }
        }
    };
    unsafe {
        StaticThread::recv_and_run(endpoint, reply_authority);
    }
}

pub fn try_idle() {
    if let Some(config) = CONFIG.get()
        && let Some(nfn) = config.idle_notification.as_ref()
    {
        sel4::cap::Notification::from_bits(nfn.to_native()).wait();
    }
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
    let addrs = CONFIG.get().unwrap().static_heap.as_ref().unwrap();
    StaticHeapBounds::new(
        usize::try_from(addrs.start).unwrap() as *mut _,
        (addrs.end - addrs.start).try_into().unwrap(),
    )
}

fn get_static_heap_mutex_notification() -> sel4::cap::Notification {
    sel4::cap::Notification::from_bits(
        CONFIG
            .get()
            .unwrap()
            .static_heap_mutex_notification
            .as_ref()
            .unwrap()
            .to_native(),
    )
}

#[cfg(not(target_arch = "arm"))]
pub fn get_backtracing() -> SimpleBacktracing {
    SimpleBacktracing::new(get_backtrace_image_identifier())
}

#[allow(dead_code)]
fn get_backtrace_image_identifier() -> Option<&'static str> {
    CONFIG
        .get()
        .unwrap()
        .image_identifier
        .as_ref()
        .map(|s| s.as_str())
}

// // //

// For macros
#[doc(hidden)]
pub mod _private {
    pub use crate::declare_main::_private as declare_main;
}
