#![no_std]
#![feature(alloc_error_handler)]
#![feature(c_size_t)]
#![feature(const_trait_impl)]
#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(linkage)]
#![feature(never_type)]
#![feature(strict_provenance)]
#![feature(thread_local)]
#![feature(unwrap_infallible)]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::ffi::{c_char, c_void};
use core::ops::Range;
use core::ptr;
use core::slice;

use sel4::Endpoint;
use sel4_backtrace_simple::SimpleBacktracing;
use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;
use sel4_panicking::ExternalPanicInfo;
use sel4_panicking_env::{abort, debug_println, AbortInfo};
use sel4_runtime_phdrs::EmbeddedProgramHeaders;
use sel4_simple_task_runtime_config_types::RuntimeConfig;
use sel4_simple_task_threading::StaticThread;

mod declare_main;
mod termination;

#[cfg(feature = "alloc")]
mod global_allocator;

pub use sel4_simple_task_runtime_macros::{main, main_json, main_postcard};

extern "Rust" {
    pub(crate) fn __sel4_simple_task_main(arg: &[u8]);
}

static CONFIG: ImmediateSyncOnceCell<RuntimeConfig<'static>> = ImmediateSyncOnceCell::new();

#[thread_local]
static THREAD_INDEX: ImmediateSyncOnceCell<usize> = ImmediateSyncOnceCell::new();

#[no_mangle]
pub unsafe extern "C" fn _start(config: *const u8, config_size: usize, thread_index: usize) -> ! {
    let config = RuntimeConfig::new(slice::from_raw_parts(config, config_size));
    let cont_arg = ContinueArg {
        config,
        thread_index,
    };
    EmbeddedProgramHeaders::finder()
        .find_tls_image()
        .reserve_on_stack_and_continue(
            cont_fn,
            (&cont_arg as *const ContinueArg)
                .cast::<c_void>()
                .cast_mut(),
        )
}

pub struct ContinueArg {
    config: RuntimeConfig<'static>,
    thread_index: usize,
}

pub unsafe extern "C" fn cont_fn(cont_arg: *mut c_void) -> ! {
    let cont_arg: &ContinueArg = &*(cont_arg.cast::<ContinueArg>().cast_const());

    let config = &cont_arg.config;
    let thread_index = cont_arg.thread_index;
    let thread_config = &config.threads()[thread_index];

    THREAD_INDEX.set(thread_index).unwrap();

    sel4::set_ipc_buffer(sel4::IPCBuffer::from_ptr(ptr::from_exposed_addr_mut(
        thread_config.ipc_buffer_addr().try_into().unwrap(),
    )));

    if thread_index == 0 {
        CONFIG.set(config.clone()).unwrap();
        sel4_runtime_phdrs::unwinding::set_custom_eh_frame_finder_using_embedded_phdrs().unwrap();
        sel4_panicking::set_hook(&panic_hook);
        __sel4_simple_task_main(config.arg());
    } else {
        StaticThread::recv_and_run(Endpoint::from_bits(thread_config.endpoint().unwrap()));
    }

    idle()
}

pub fn idle() -> ! {
    if let Some(nfn) = CONFIG
        .get()
        .unwrap()
        .idle_notification()
        .map(sel4::Notification::from_bits)
    {
        nfn.wait();
    }
    abort!("idle failed")
}

#[no_mangle]
fn sel4_runtime_abort_hook(info: Option<&AbortInfo>) {
    match info {
        Some(info) => debug_println!("{}", info),
        None => debug_println!("(aborted)"),
    }
    idle()
}

#[no_mangle]
fn sel4_runtime_debug_put_char(c: c_char) {
    sel4::debug_put_char(c)
}

fn panic_hook(info: &ExternalPanicInfo<'_>) {
    debug_println!("{}", info);
    get_backtracing().collect_and_send();
}

fn get_static_heap_bounds() -> Range<*mut u8> {
    let addrs = CONFIG.get().unwrap().static_heap().unwrap();
    ptr::from_exposed_addr_mut(addrs.start.try_into().unwrap())
        ..ptr::from_exposed_addr_mut(addrs.end.try_into().unwrap())
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
