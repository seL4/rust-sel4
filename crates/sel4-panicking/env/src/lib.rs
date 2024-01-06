//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(core_intrinsics)]
#![feature(linkage)]
#![allow(internal_features)]

use core::fmt;
use core::panic::Location;
use core::str;

extern "Rust" {
    fn sel4_runtime_abort_hook(info: Option<&AbortInfo>);
    fn sel4_runtime_debug_put_char(c: u8);
}

mod defaults {
    use super::{default_abort_hook, AbortInfo};

    #[no_mangle]
    #[linkage = "weak"]
    fn sel4_runtime_abort_hook(info: Option<&AbortInfo>) {
        default_abort_hook(info)
    }
}

// // //

/// Prints via a link-time hook.
///
/// This function uses the following externally defined symobol:
///
/// ```rust
/// extern "Rust" {
///     fn sel4_runtime_debug_put_char(c: u8);
/// }
/// ```
pub fn debug_put_char(c: u8) {
    unsafe { sel4_runtime_debug_put_char(c) }
}

struct DebugWrite;

impl fmt::Write for DebugWrite {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &c in s.as_bytes() {
            debug_put_char(c)
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn debug_print_helper(args: fmt::Arguments) {
    fmt::write(&mut DebugWrite, args).unwrap_or_else(|err| {
        // Just report error. This this function must not fail.
        let _ = fmt::write(&mut DebugWrite, format_args!("({err})"));
    })
}

/// Like `std::print`, except backed by [`debug_put_char`].
#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => ($crate::debug_print_helper(format_args!($($arg)*)));
}

/// Like `std::println`, except backed by [`debug_put_char`].
#[macro_export]
macro_rules! debug_println {
    () => ($crate::debug_println!(""));
    ($($arg:tt)*) => ({
        $crate::debug_print!($($arg)*);
        $crate::debug_print!("\n");
    })
}

// // //

/// Information about an abort passed to an abort hook.
pub struct AbortInfo<'a> {
    message: Option<&'a fmt::Arguments<'a>>,
    location: Option<&'a Location<'a>>,
}

impl<'a> AbortInfo<'a> {
    /// The `core::fmt::Arguments` with which [`abort!`] was called.
    pub fn message(&self) -> Option<&fmt::Arguments> {
        self.message
    }

    /// The location at which [`abort!`] was called.
    pub fn location(&self) -> Option<&Location> {
        self.location
    }
}

impl fmt::Display for AbortInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("aborted at ")?;
        if let Some(message) = self.message {
            write!(f, "'{message}', ")?;
        }
        if let Some(location) = self.location {
            location.fmt(f)?;
        } else {
            write!(f, "unknown location")?;
        }
        Ok(())
    }
}

fn abort(info: Option<&AbortInfo>) -> ! {
    unsafe {
        sel4_runtime_abort_hook(info);
    }
    core::intrinsics::abort()
}

fn default_abort_hook(info: Option<&AbortInfo>) {
    match info {
        Some(info) => debug_println!("{}", info),
        None => debug_println!("(aborted)"),
    }
}

/// Abort without any [`AbortInfo`].
///
/// This function does the same thing as [`abort!`], except it passes `None` to the abort hook.
pub fn abort_without_info() -> ! {
    abort(None)
}

#[doc(hidden)]
#[track_caller]
pub fn abort_helper(args: fmt::Arguments) -> ! {
    abort(Some(&AbortInfo {
        message: Some(&args),
        location: Some(Location::caller()),
    }))
}

/// Abort execution with a message.
///
/// This function first invokes an externally defined abort hook which is resolved at link time,
/// and then calls `core::intrinsics::abort()`.
#[macro_export]
macro_rules! abort {
    () => ($crate::abort!(""));
    ($($arg:tt)*) => ($crate::abort_helper(format_args!($($arg)*)));
}
