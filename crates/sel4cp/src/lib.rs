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

//! A foundation for pure-Rust [seL4 Core Platform](https://github.com/BreakawayConsulting/sel4cp)
//! protection domains.
//!
//! See the [seL4 Core Platform
//! manual](https://github.com/BreakawayConsulting/sel4cp/blob/main/docs/manual.md) for
//! non-Rust-specific documentation about the seL4 Core Platform.
//!
//! See [the demo](https://gitlab.com/coliasgroup/rust-seL4-demos/simple-sel4cp-demo/) for a
//! concrete example of this crate in action.
//!
//! This crate depends, at build time, on the libsel4 headers. It requires that either
//! `$SEL4_INCLUDE_DIRS` contains a colon-separated list of include paths for the libsel4 headers,
//! or that `$SEL4_PREFIX` is set, in which case `$SEL4_PREFIX/libsel4/include` is used.
//!
//! The `sel4cp` tool expects protection domain binaries to expose a few symbols. All protection
//! domains must contain the symbol `__sel4_ipc_buffer_obj`. Furthermore, for protection domains
//! with memory regions, the `sel4cp` tool injects the addresses of these memory regions at build
//! time by patching designated symbols. The
//! [`*-sel4cp{,-minimal}.json`](https://gitlab.com/coliasgroup/rust-seL4/-/tree/main/support/targets)
//! `rustc` target specs distributed as part of the [rust-seL4
//! project](https://gitlab.com/coliasgroup/rust-seL4) provide `__sel4_ipc_buffer_obj`, and the
//! [`memory_region_symbol`] macro provides a conveneint way to declare memory region address
//! symbols.
//!
//! Use the [`protection_domain`] macro to declare the initialization function, stack size, and,
//! optionally, heap and heap size.

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
#[doc(hidden)]
pub const DEFAULT_STACK_SIZE: usize = 0x10000;

/// Declares the initialization function, stack size, and, optionally, heap and heap size.
#[macro_export]
macro_rules! declare_protection_domain {
    (init = $init:path $(,)?) => {
        $crate::_private::declare_protection_domain!(init = $init, stack_size = $crate::_private::DEFAULT_STACK_SIZE);
    };
    (init = $init:path, stack_size = $stack_size:expr $(,)?) => {
        $crate::_private::declare_init!($init);
        $crate::_private::declare_stack!($stack_size);
    };
    (init = $init:path, $(stack_size = $stack_size:expr,)? heap_size = $heap_size:expr $(,)?) => {
        $crate::_private::declare_static_heap! {
            __GLOBAL_ALLOCATOR: $heap_size;
        }
        $crate::_private::declare_protection_domain!(init = $init $(, stack_size = $stack_size)?);
    };
}

// For macros
#[doc(hidden)]
pub mod _private {
    pub use crate::{declare_init, declare_protection_domain, entry::run_main, DEFAULT_STACK_SIZE};

    pub use sel4::sys::seL4_BootInfo;
    pub use sel4_runtime_common::declare_stack;
    pub use sel4_runtime_common::declare_static_heap;
}

sel4::config::sel4_cfg_if! {
    if #[cfg(PRINTING)] {
        pub use sel4_panicking_env::{debug_print, debug_println};
    } else {
        /// No-op for this configuration.
        #[macro_export]
        macro_rules! debug_print {
            ($($arg:tt)*) => {};
        }

        /// No-op for this configuration.
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

pub(crate) unsafe fn get_ipc_buffer() -> sel4::IPCBuffer {
    sel4::IPCBuffer::from_ptr(&mut __sel4_ipc_buffer_obj)
}

#[no_mangle]
#[link_section = ".data"]
static mut passive: bool = false; // just a placeholder

/// Returns whether this projection domain is passive.
pub fn pd_is_passive() -> bool {
    unsafe { passive }
}

#[no_mangle]
#[link_section = ".data"]
static sel4cp_name: [u8; 16] = [0; 16];

/// Returns the name of this projection domain.
pub fn pd_name() -> &'static str {
    // avoid recursive panic
    fn on_err<T, U>(_: T) -> U {
        abort!("invalid embedded protection domain name");
    }
    core::ffi::CStr::from_bytes_until_nul(&sel4cp_name)
        .unwrap_or_else(&on_err)
        .to_str()
        .unwrap_or_else(&on_err)
}
