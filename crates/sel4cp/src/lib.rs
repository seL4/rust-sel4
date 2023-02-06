#![no_std]
#![feature(cstr_from_bytes_until_nul)]
#![feature(core_intrinsics)]
#![feature(exclusive_wrapper)]
#![feature(unwrap_infallible)]
#![feature(never_type)]
#![feature(strict_provenance)]

extern crate sel4_runtime_simple_entry;

#[cfg(feature = "global-allocator")]
extern crate sel4_runtime_simple_static_heap;

use core::fmt;

pub use sel4cp_macros::main;

#[cfg(feature = "global-allocator")]
pub use sel4_runtime_simple_static_heap::set_mutex_notification as set_heap_mutex_notification;

mod channel;
mod handler;
mod ipc_buffer;
mod pd_name;

pub use channel::*;
pub use handler::*;

use ipc_buffer::get_ipc_buffer;
pub use pd_name::get_pd_name;

#[macro_export]
macro_rules! declare_main {
    ($main:path) => {
        #[no_mangle]
        pub extern "C" fn __rust_entry() -> ! {
            $crate::_private::run_main($main)
        }
    };
}

pub fn run_main<T>(f: impl FnOnce() -> T) -> !
where
    T: Handler,
    T::Error: fmt::Debug,
{
    unsafe {
        sel4::set_ipc_buffer(get_ipc_buffer());
    }

    let err = f().run().into_err();

    sel4::debug_println!("Terminated with error: {:?}", err);
    abort()
}

#[cfg(panic = "unwind")]
compile_error!("");

#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    sel4::debug_println!("({}) {}", get_pd_name(), info);
    abort()
}

fn abort() -> ! {
    sel4::debug_println!("(aborting)");
    core::intrinsics::abort()
}

#[doc(hidden)]
pub mod _private {
    pub use crate::run_main;
}
