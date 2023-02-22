#![no_std]
#![feature(cfg_target_thread_local)]
#![feature(const_pointer_is_aligned)]
#![feature(const_ptr_is_null)]
#![feature(const_trait_impl)]
#![feature(cstr_from_bytes_until_nul)]
#![feature(int_roundings)]
#![feature(maybe_uninit_slice)]
#![feature(never_type)]
#![feature(pointer_is_aligned)]
#![feature(unwrap_infallible)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use sel4_panicking_env::{abort, debug_print, debug_println};
pub use sel4cp_macros::main;

mod cspace;
mod entry;
mod handler;
mod ipc_buffer;
mod pd_name;

pub mod memory_region;
pub mod message;
pub mod panicking;

pub use cspace::{Channel, IrqAckError};
pub use handler::{Handler, NullHandler};
pub use pd_name::get_pd_name;

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
    pub use sel4_runtime_simple_entry::declare_stack;
    pub use sel4_runtime_simple_static_heap::declare_static_heap;
}
