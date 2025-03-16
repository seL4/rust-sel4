//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(linkage)]

use core::fmt;
use core::panic::Location;
use core::str;

extern "Rust" {
    fn __sel4_panicking_env__debug_put_char(c: u8);
    fn __sel4_panicking_env__abort_hook(info: Option<&AbortInfo>);
    fn __sel4_panicking_env__abort_trap() -> !;
}

/// Registers a function to be used by [`debug_put_char`], [`debug_print!`], and [`debug_println!`].
///
/// This macro uses the function `$path` to define the following symbol:
///
/// ```rust
/// extern "Rust" {
///     fn __sel4_panicking_env__debug_put_char(c: u8);
/// }
/// ```
#[macro_export]
macro_rules! register_debug_put_char {
    ($(#[$attrs:meta])* $path:path) => {
        #[allow(non_snake_case)]
        const _: () = {
            $(#[$attrs])*
            #[no_mangle]
            fn __sel4_panicking_env__debug_put_char(c: u8) {
                const F: fn(u8) = $path;
                F(c)
            }
        };
    };
}

/// Registers an abort hook to be used by [`abort!`] and [`abort_without_info`].
///
/// This macro uses the function `$path` to define the following symbol:
///
/// ```rust
/// extern "Rust" {
///     fn __sel4_panicking_env__abort_hook(info: Option<&AbortInfo>);
/// }
/// ```
#[macro_export]
macro_rules! register_abort_hook {
    ($(#[$attrs:meta])* $path:path) => {
        #[allow(non_snake_case)]
        const _: () = {
            $(#[$attrs])*
            #[no_mangle]
            fn __sel4_panicking_env__abort_hook(info: ::core::option::Option<&$crate::AbortInfo>) {
                const F: fn(::core::option::Option<&$crate::AbortInfo>) = $path;
                F(info)
            }
        };
    };
}

register_abort_hook!(
    #[linkage = "weak"]
    default_abort_hook
);

fn default_abort_hook(info: Option<&AbortInfo>) {
    match info {
        Some(info) => debug_println!("{}", info),
        None => debug_println!("(aborted)"),
    }
}

/// Registers an abort trap to be used by [`abort!`] and [`abort_without_info`].
///
/// This macro uses the function `$path` to define the following symbol:
///
/// ```rust
/// extern "Rust" {
///     fn __sel4_panicking_env__abort_trap() -> !;
/// }
/// ```
#[macro_export]
macro_rules! register_abort_trap {
    ($(#[$attrs:meta])* $path:path) => {
        #[allow(non_snake_case)]
        const _: () = {
            $(#[$attrs])*
            #[no_mangle]
            fn __sel4_panicking_env__abort_trap() -> ! {
                const F: fn() -> ! = $path;
                F()
            }
        };
    };
}

// // //

/// Prints via a link-time hook.
///
/// This function uses the following externally defined symbol:
///
/// ```rust
/// extern "Rust" {
///     fn __sel4_panicking_env__debug_put_char(c: u8);
/// }
/// ```
///
/// [`register_debug_put_char`] provides a typesafe way to define that symbol.
pub fn debug_put_char(c: u8) {
    unsafe { __sel4_panicking_env__debug_put_char(c) }
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
pub fn __debug_print_macro_helper(args: fmt::Arguments) {
    fmt::write(&mut DebugWrite, args).unwrap_or_else(|err| {
        // Just report error. This function must not fail.
        let _ = fmt::write(&mut DebugWrite, format_args!("({err})"));
    })
}

/// Like `std::print!`, except backed by [`debug_put_char`].
#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => ($crate::__debug_print_macro_helper(format_args!($($arg)*)));
}

/// Like `std::println!`, except backed by [`debug_put_char`].
#[macro_export]
macro_rules! debug_println {
    () => ($crate::debug_println!(""));
    ($($arg:tt)*) => ($crate::debug_print!("{}\n", format_args!($($arg)*)));
}

// // //

/// Information about an abort passed to an abort hook.
pub struct AbortInfo<'a> {
    message: Option<&'a fmt::Arguments<'a>>,
    location: Option<&'a Location<'a>>,
}

impl AbortInfo<'_> {
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
        if let Some(location) = self.location {
            location.fmt(f)?;
        } else {
            f.write_str("unknown location")?;
        }
        if let Some(message) = self.message {
            f.write_str(":\n")?;
            f.write_fmt(*message)?;
        }
        Ok(())
    }
}

fn abort(info: Option<&AbortInfo>) -> ! {
    unsafe {
        __sel4_panicking_env__abort_hook(info);
        __sel4_panicking_env__abort_trap()
    }
}

/// Aborts without any [`AbortInfo`].
///
/// This function does the same thing as [`abort!`], except it passes `None` to the abort hook.
pub fn abort_without_info() -> ! {
    abort(None)
}

#[doc(hidden)]
#[track_caller]
pub fn __abort_macro_helper(message: Option<fmt::Arguments>) -> ! {
    abort(Some(&AbortInfo {
        message: message.as_ref(),
        location: Some(Location::caller()),
    }))
}

/// Aborts execution with a message.
///
/// [`abort!`] accepts the same patterns `core::panic!`:
///
/// ```rust
/// abort!();
/// abort!("uh oh!");
/// abort!("uh {} {}!", 123, "oh");
/// ```
///
/// This macro first invokes an externally defined abort hook which is resolved at link time, and
/// then calls `core::intrinsics::abort()`.
///
/// The following externally defined symbol is used as the abort hook:
///
/// ```rust
/// extern "Rust" {
///     fn __sel4_panicking_env__abort_hook(info: Option<&AbortInfo>);
/// }
/// ```
///
/// The [`sel4_panicking_env` crate](crate) defines a weak version of this symbol which just prints
/// the [`AbortInfo`] argument using [`debug_print!`].
///
/// [`register_abort_hook`] provides a typesafe way to define that symbol.
#[macro_export]
macro_rules! abort {
    () => ($crate::__abort_macro_helper(::core::option::Option::None));
    ($($arg:tt)*) => ($crate::__abort_macro_helper(::core::option::Option::Some(format_args!($($arg)*))));
}
