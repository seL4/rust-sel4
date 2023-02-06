#![no_std]
#![feature(core_intrinsics)]
#![feature(const_trait_impl)]
#![feature(alloc_error_handler)]
#![feature(unwrap_infallible)]
#![feature(never_type)]
#![feature(strict_provenance)]
#![feature(lang_items)]

extern crate sel4_runtime_simple_entry;

#[cfg(feature = "global-allocator")]
extern crate sel4_runtime_simple_static_heap;

use core::ffi::c_char;

mod start;

#[cfg(feature = "unwinding")]
mod unwinding;

pub mod _private {
    pub use crate::start::_private as start;
}

// re-exports

pub use sel4_full_root_task_runtime_macros::main;
pub use sel4_panicking::catch_unwind;
pub use sel4_panicking_env::{abort, debug_print, debug_println};
pub use sel4_runtime_simple_termination::Termination;

//

#[no_mangle]
fn sel4_runtime_debug_put_char(c: c_char) {
    sel4::debug_put_char(c)
}
