//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;

pub use sel4_never_polyfill::Never;

/// Trait for the return type of [`#[root_task]`](crate::root_task) main functions.
pub trait Termination {
    type Error: fmt::Debug;

    fn report(self) -> Self::Error;
}

impl Termination for Never {
    type Error = Never;

    fn report(self) -> Self::Error {
        self
    }
}

impl<E: fmt::Debug> Termination for Result<Never, E> {
    type Error = E;

    fn report(self) -> Self::Error {
        match self {
            #[allow(unreachable_patterns)]
            Ok(absurdity) => match absurdity {},
            Err(err) => err,
        }
    }
}
