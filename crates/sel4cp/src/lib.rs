#![no_std]
#![feature(cfg_target_thread_local)]
#![feature(const_pointer_is_aligned)]
#![feature(const_ptr_is_null)]
#![feature(const_trait_impl)]
#![feature(int_roundings)]
#![feature(maybe_uninit_slice)]
#![feature(never_type)]
#![feature(pointer_is_aligned)]
#![feature(proc_macro_hygiene)]
#![feature(stmt_expr_attributes)]
#![feature(unwrap_infallible)]

#[cfg(feature = "alloc")]
extern crate alloc;

use sel4_panicking_env::abort;

pub use sel4cp_macros::protection_domain;

mod cspace;
mod entry;
mod handler;

pub mod memory_region;
pub mod message;
pub mod panicking;

pub use cspace::{Channel, DeferredAction, IrqAckError};
pub use handler::{Handler, NullHandler};

// TODO decrease
pub const DEFAULT_STACK_SIZE: usize = 0x10000;

#[macro_export]
macro_rules! declare_protection_domain {
    ($main:path) => {
        $crate::_private::declare_protection_domain!($main, stack_size = $crate::_private::DEFAULT_STACK_SIZE);
    };
    ($main:path, stack_size = $stack_size:expr) => {
        $crate::_private::declare_main!($main);
        $crate::_private::declare_stack!($stack_size);
    };
    ($main:path, $(stack_size = $stack_size:expr,)? heap_size = $heap_size:expr) => {
        $crate::_private::declare_static_heap! {
            __GLOBAL_ALLOCATOR: $heap_size;
        }
        $crate::_private::declare_protection_domain!($main $(, stack_size = $stack_size)?);
    };
}

// For macros
#[doc(hidden)]
pub mod _private {
    pub use crate::{declare_main, declare_protection_domain, entry::run_main, DEFAULT_STACK_SIZE};

    pub use sel4::sys::seL4_BootInfo;
    pub use sel4_runtime_common::declare_stack;
    pub use sel4_runtime_common::declare_static_heap;
}

sel4::config::sel4_cfg_if! {
    if #[cfg(PRINTING)] {
        pub use sel4_panicking_env::{debug_print, debug_println};
    } else {
        #[macro_export]
        macro_rules! debug_print {
            ($($arg:tt)*) => {};
        }

        #[macro_export]
        macro_rules! debug_println {
            ($($arg:tt)*) => {};
        }
    }
}

// // //

extern "C" {
    static mut __sel4_ipc_buffer_obj: sel4::sys::seL4_IPCBuffer;
}

pub unsafe fn get_ipc_buffer() -> sel4::IPCBuffer {
    sel4::IPCBuffer::from_ptr(&mut __sel4_ipc_buffer_obj)
}

#[no_mangle]
#[link_section = ".data"]
static mut passive: bool = false; // just a placeholder

pub fn is_passive() -> bool {
    unsafe { passive }
}

#[no_mangle]
#[link_section = ".data"]
static sel4cp_name: [u8; 16] = [0; 16];

pub fn get_pd_name() -> &'static str {
    // avoid recursive panic
    fn on_err<T, U>(_: T) -> U {
        abort!("invalid embedded protection domain name");
    }
    core::ffi::CStr::from_bytes_until_nul(&sel4cp_name)
        .unwrap_or_else(&on_err)
        .to_str()
        .unwrap_or_else(&on_err)
}
