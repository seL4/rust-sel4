//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

use alloc::format;
use alloc::string::String;
use core::fmt;

use sel4_never_polyfill::Never;

use crate::{config::get_config, run_tests::run_tests_with_config};

pub(crate) mod types;

pub use types::*;

pub trait Termination {
    type Error: fmt::Debug;

    fn report(self) -> Result<(), Self::Error>;
}

impl Termination for () {
    type Error = Never;

    fn report(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Termination for Never {
    type Error = Never;

    fn report(self) -> Result<(), Self::Error> {
        self
    }
}

impl<T, E: fmt::Debug> Termination for Result<T, E> {
    type Error = E;

    fn report(self) -> Result<(), Self::Error> {
        self.map(|_| ())
    }
}

pub fn assert_test_result<T: Termination>(result: T) -> Result<(), String> {
    result.report().map_err(|err| {
        format!("the test returned a termination value of Err({err:?}) which indicates a failure")
    })
}

// Used by older toolchains
pub fn test_main_static(tests: &[&TestDescAndFn]) {
    run_tests_with_config(&get_config(), tests)
}

pub type ExitCode = ();

pub fn test_main_env_args(tests: &[&TestDescAndFn]) -> ExitCode {
    test_main_static(tests)
}

pub fn test_main_env_args_abort(tests: &[&TestDescAndFn]) -> ExitCode {
    test_main_static(tests)
}
