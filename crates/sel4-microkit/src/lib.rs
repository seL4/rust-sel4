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
#![feature(slice_ptr_get)]
#![feature(stmt_expr_attributes)]
#![feature(unwrap_infallible)]
#![feature(used_with_arg)]

//! A foundation for pure-Rust [seL4 Microkit](https://github.com/seL4/microkit) protection domains.
//!
//! See the [seL4 Microkit manual](https://github.com/seL4/microkit/blob/main/docs/manual.md) for
//! non-Rust-specific documentation about the seL4 Microkit.
//!
//! See [the demo](https://github.com/coliasgroup/rust-root-task-demo) for a concrete example of
//! this crate in action.
//!
//! This crate depends, at build time, on the libsel4 headers. It requires that either
//! `$SEL4_INCLUDE_DIRS` contains a colon-separated list of include paths for the libsel4 headers,
//! or that `$SEL4_PREFIX` is set, in which case `$SEL4_PREFIX/libsel4/include` is used.
//!
//! The `microkit` tool expects protection domain binaries to expose a few symbols. All protection
//! domains must contain the symbol `__sel4_ipc_buffer_obj`. Furthermore, for protection domains
//! with memory regions, the `microkit` tool injects the addresses of these memory regions at build
//! time by patching designated symbols. The
//! [`*-sel4-microkit{,-minimal}.json`](https://github.com/coliasgroup/rust-sel4/tree/main/support/targets)
//! `rustc` target specs distributed as part of the [rust-sel4
//! project](https://github.com/coliasgroup/rust-sel4) provide `__sel4_ipc_buffer_obj`, and the
//! [`memory_region_symbol`] macro provides a conveneint way to declare memory region address
//! symbols.
//!
//! Use the [`protection_domain`] macro to declare the initialization function, stack size, and,
//! optionally, heap and heap size.

#[cfg(feature = "alloc")]
extern crate alloc;

use sel4_panicking_env::abort;

pub use sel4_microkit_macros::protection_domain;

mod cspace;
mod entry;
mod env;
mod handler;
mod memory_region;
mod message;

pub mod panicking;

pub use cspace::{
    Channel, DeferredAction, DeferredActionInterface, DeferredActionSlot, IrqAckError,
};
pub use env::{pd_is_passive, pd_name};
pub use handler::{Handler, NullHandler};
pub use memory_region::{cast_memory_region_checked, cast_memory_region_to_slice_checked};
pub use message::{
    get_mr, set_mr, with_msg_bytes, with_msg_bytes_mut, with_msg_regs, with_msg_regs_mut,
    MessageInfo, MessageLabel, MessageRegisterValue,
};

/// Declares the initialization function, stack size, and, optionally, heap and heap size.
///
/// See the [`protection_domain`] attribute macro for more detail.
#[macro_export]
macro_rules! declare_protection_domain {
     {
        init = $init:expr $(,)?
    } => {
        $crate::_private::declare_protection_domain! {
            init = $init,
            stack_size = $crate::_private::DEFAULT_STACK_SIZE,
        }
    };
    {
        init = $init:expr,
        stack_size = $stack_size:expr $(,)?
    } => {
        $crate::_private::declare_init!($init);
        $crate::_private::declare_stack!($stack_size);
    };
    {
        init = $init:expr,
        $(stack_size = $stack_size:expr,)?
        heap_size = $heap_size:expr $(,)?
    } => {
        $crate::_private::declare_static_heap! {
            #[doc(hidden)]
            __GLOBAL_ALLOCATOR: $heap_size;
        }
        $crate::_private::declare_protection_domain! {
            init = $init,
            $(stack_size = $stack_size,)?
        }
    };
}

// For macros
#[doc(hidden)]
pub mod _private {
    pub use sel4_immutable_cell::ImmutableCell;

    pub use sel4_runtime_common::{declare_stack, declare_static_heap};

    pub use crate::{declare_init, declare_protection_domain, entry::run_main};

    pub const DEFAULT_STACK_SIZE: usize = 0x10000;
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
