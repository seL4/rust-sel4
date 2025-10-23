//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;
use core::panic::UnwindSafe;

use serde::Deserialize;

use sel4_panicking::catch_unwind;
use sel4_panicking_env::debug_println;

use crate::termination::Termination;

pub fn run_main<T: Termination>(f: impl Fn(&[u8]) -> T, arg: &[u8])
where
    T::Error: fmt::Debug,
{
    f(arg).show()
}

#[cfg(feature = "serde_json")]
pub fn run_main_json<T: Termination, U: for<'a> Deserialize<'a>>(f: impl Fn(U) -> T, arg: &[u8])
where
    T::Error: fmt::Debug,
{
    match serde_json::from_slice(arg) {
        Ok(arg) => f(arg).show(),
        Err(err) => {
            debug_println!("failed to deserialize arg: {}", err)
        }
    }
}

pub fn run_main_postcard<T: Termination, U: for<'a> Deserialize<'a>>(f: impl Fn(U) -> T, arg: &[u8])
where
    T::Error: fmt::Debug,
{
    match postcard::from_bytes(arg) {
        Ok(arg) => f(arg).show(),
        Err(err) => {
            debug_println!("failed to deserialize arg: {}", err)
        }
    }
}

pub fn wrap(f: impl FnOnce() + UnwindSafe) {
    let _ = catch_unwind(|| {
        f();
    });
}

#[macro_export]
macro_rules! declare_main_with {
    ($f:ident, $main:path) => {
        #[unsafe(no_mangle)]
        pub fn __sel4_simple_task_main(arg: &[u8]) {
            $crate::_private::declare_main::wrap(|| {
                $crate::_private::declare_main::$f($main, arg);
            })
        }
    };
}

pub mod _private {
    pub use super::run_main;
    pub use super::run_main_postcard;
    pub use super::wrap;

    #[cfg(feature = "serde_json")]
    pub use super::run_main_json;
}
