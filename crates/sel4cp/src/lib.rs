#![no_std]
#![feature(cfg_target_thread_local)]
#![feature(cstr_from_bytes_until_nul)]
#![feature(int_roundings)]
#![feature(never_type)]
#![feature(unwrap_infallible)]

extern crate sel4_runtime_simple_entry;

#[cfg(feature = "global-allocator")]
extern crate sel4_runtime_simple_static_heap;

use core::ffi::c_char;
use core::fmt;

#[cfg(target_thread_local)]
use core::ffi::c_void;

#[cfg(target_thread_local)]
use core::ptr;

#[cfg(target_thread_local)]
use sel4_runtime_phdrs::EmbeddedProgramHeaders;

pub use sel4_panicking as panicking;
pub use sel4_panicking_env::{abort, debug_print, debug_println};
pub use sel4cp_macros::main;

#[cfg(feature = "global-allocator")]
pub use sel4_runtime_simple_static_heap::GLOBAL_ALLOCATOR;

mod channel;
mod handler;
mod ipc_buffer;
mod pd_name;

use ipc_buffer::get_ipc_buffer;

pub use channel::*;
pub use handler::*;
pub use pd_name::get_pd_name;

#[cfg(target_thread_local)]
#[no_mangle]
unsafe extern "C" fn sel4_runtime_rust_entry() -> ! {
    unsafe extern "C" fn cont_fn(_cont_arg: *mut c_void) -> ! {
        inner_entry()
    }

    let cont_arg = ptr::null_mut();

    EmbeddedProgramHeaders::finder()
        .find_tls_image()
        .reserve_on_stack_and_continue(cont_fn, cont_arg)
}

#[cfg(not(target_thread_local))]
#[no_mangle]
unsafe extern "C" fn sel4_runtime_rust_entry() -> ! {
    inner_entry()
}

unsafe extern "C" fn inner_entry() -> ! {
    #[cfg(feature = "unwinding")]
    {
        sel4_runtime_phdrs::unwinding::set_custom_eh_frame_finder_using_embedded_phdrs().unwrap();
    }

    panicking::set_hook(&panic_hook);
    sel4::set_ipc_buffer(get_ipc_buffer());
    __sel4cp_main();
    abort!("main thread returned")
}

fn panic_hook(info: &panicking::ExternalPanicInfo) {
    debug_println!("{}: {}", get_pd_name(), info);
}

extern "C" {
    fn __sel4cp_main();
}

#[macro_export]
macro_rules! declare_main {
    ($main:path) => {
        #[no_mangle]
        pub unsafe extern "C" fn __sel4cp_main() {
            $crate::_private::run_main($main);
        }
    };
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn run_main<T>(f: impl FnOnce() -> T)
where
    T: Handler,
    T::Error: fmt::Debug,
{
    match panicking::catch_unwind(|| f().run().into_err()) {
        Ok(err) => abort!("main thread terminated with error: {err:?}"),
        Err(_) => abort!("main thread panicked"),
    }
}

#[no_mangle]
fn sel4_runtime_debug_put_char(c: c_char) {
    sel4::debug_put_char(c)
}

// For macros
#[doc(hidden)]
pub mod _private {
    pub use super::run_main;
    pub use sel4::sys::seL4_BootInfo;
}
