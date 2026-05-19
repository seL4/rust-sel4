//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(unused_imports)]
#![allow(unused_macros)]

use core::fmt;

use crate::plat::{Plat, PlatImpl};

struct DebugWrite;

impl fmt::Write for DebugWrite {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &c in s.as_bytes() {
            PlatImpl::put_char(c);
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
    () => ($crate::fmt::debug_println!(""));
    ($($arg:tt)*) => ($crate::fmt::debug_print!("{}\n", format_args!($($arg)*)));
}

pub(crate) use debug_print;
pub(crate) use debug_println;
