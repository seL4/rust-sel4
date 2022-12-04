use core::fmt;

use crate::debug;

struct DebugWrite;

impl fmt::Write for DebugWrite {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &c in s.as_bytes() {
            debug::put_char(c);
        }
        Ok(())
    }
}

pub(crate) fn debug_print_helper(args: fmt::Arguments) {
    fmt::write(&mut DebugWrite, args).unwrap_or_else(|err| panic!("write error: {:?}", err))
}

macro_rules! debug_print {
    ($($arg:tt)*) => ($crate::fmt::debug_print_helper(format_args!($($arg)*)));
}

macro_rules! debug_println {
    () => ($crate::debug_print!("\n"));
    ($($arg:tt)*) => ({
        $crate::debug_print!($($arg)*);
        $crate::debug_print!("\n");
    })
}

pub(crate) use debug_print;
pub(crate) use debug_println;
