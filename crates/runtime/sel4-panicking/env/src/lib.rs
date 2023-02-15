#![no_std]
#![feature(core_intrinsics)]
#![feature(linkage)]

use core::ffi::c_char;
use core::fmt;
use core::str;

extern "Rust" {
    pub(crate) fn sel4_runtime_abort_hook();
    pub(crate) fn sel4_runtime_debug_put_char(c: c_char);
}

pub fn abort() -> ! {
    unsafe {
        sel4_runtime_abort_hook();
    }
    debug_println!("(aborting)");
    core::intrinsics::abort()
}

pub fn debug_put_char(c: c_char) {
    unsafe { sel4_runtime_debug_put_char(c) }
}

mod defaults {
    #[no_mangle]
    #[linkage = "weak"]
    fn sel4_runtime_abort_hook() {}
}

pub struct DebugWrite;

impl fmt::Write for DebugWrite {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &c in s.as_bytes() {
            debug_put_char(c as c_char)
        }
        Ok(())
    }
}

// TODO
// Report location with #[track_caller]
#[doc(hidden)]
pub fn abort_helper(args: fmt::Arguments) -> ! {
    debug_println!("{}", args);
    abort()
}

#[macro_export]
macro_rules! abort {
    ($($arg:tt)*) => ($crate::abort_helper(format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn debug_print_helper(args: fmt::Arguments) {
    fmt::write(&mut DebugWrite, args).unwrap_or_else(|err| {
        // NOTE
        // Possibility of panic-in-panic. I think it is best to rely on any downstream
        // #[panic_handler]'s panic-within-panic handling rather than making an opinionated choice
        // of a lower-level abort mechanism here.
        // TODO
        // Add try_debug_print{,ln} alternatives.
        panic!("write error: {:?}", err)
    })
}

#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => ($crate::debug_print_helper(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! debug_println {
    () => ($crate::debug_print!("\n"));
    ($($arg:tt)*) => ({
        $crate::debug_print!($($arg)*);
        $crate::debug_print!("\n");
    })
}
