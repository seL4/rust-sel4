use core::ffi::c_char;
use core::fmt;

use crate::debug_put_char;

#[doc(hidden)]
pub mod _private {
    pub use super::debug_print_helper;
}

struct Debug;

impl fmt::Write for Debug {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &c in s.as_bytes() {
            debug_put_char(c as c_char)
        }
        Ok(())
    }
}

pub fn debug_print_helper(args: fmt::Arguments) {
    fmt::write(&mut Debug, args).unwrap_or_else(|err| {
        // NOTE(nspin)
        // If a runtime's #[panic_handler] uses this debug_print{ln}, then this
        // would result in a panic-within-panic. I think it is best to rely
        // on any downstream #[panic_handler]'s panic-within-panic handling
        // rather than making an opinionated choice of a lower-level abort
        // mechanism here.
        panic!("write error: {:?}", err)
    })
}

#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => ($crate::_private::fmt::debug_print_helper(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! debug_println {
    () => ($crate::debug_print!("\n"));
    ($($arg:tt)*) => ({
        // NOTE
        // If #[feature(format_args_nl)] is ever stabilized, replace with:
        // $crate::_private::fmt::debug_print_helper(format_args_nl!($($arg)*));
        $crate::debug_print!($($arg)*);
        $crate::debug_print!("\n");
    })
}
