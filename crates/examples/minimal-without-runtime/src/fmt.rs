use core::ffi::c_char;
use core::fmt;

pub struct Debug;

impl fmt::Write for Debug {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &c in s.as_bytes() {
            sel4_sys::seL4_DebugPutChar(c as c_char)
        }
        Ok(())
    }
}

pub(crate) fn debug_print_helper(args: fmt::Arguments) {
    fmt::write(&mut Debug, args).unwrap()
}

macro_rules! debug_print {
    ($($arg:tt)*) => ($crate::fmt::debug_print_helper(format_args!($($arg)*)));
}

pub(crate) use debug_print;
