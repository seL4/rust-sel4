#![no_std]
#![feature(core_intrinsics)]
#![feature(const_trait_impl)]
#![feature(ptr_to_from_bits)]
#![feature(alloc_error_handler)]
#![feature(unwrap_infallible)]
#![feature(never_type)]
#![feature(lang_items)]

extern crate sel4_runtime_building_blocks_root_task_head;

use core::ffi::c_char;

mod start;

#[cfg(feature = "global-allocator")]
mod global_allocator;

#[cfg(feature = "unwinding")]
mod unwinding;

#[cfg(all(feature = "unwinding", feature = "postcard"))]
pub mod backtrace;

pub mod _private {
    pub use crate::start::_private as start;
}

// re-exports

pub use sel4_full_root_task_runtime_macros::main;
pub use sel4_panicking::catch_unwind;
pub use sel4_runtime_building_blocks_abort::{abort, debug_print, debug_println};
pub use sel4_runtime_building_blocks_termination::Termination;

//

fn panic_hook() {
    #[cfg(all(feature = "unwinding", feature = "postcard"))]
    backtrace::collect_and_send();
}

#[no_mangle]
fn sel4_runtime_debug_put_char(c: c_char) {
    sel4::debug_put_char(c)
}
