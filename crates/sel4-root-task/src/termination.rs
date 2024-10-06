//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;

/// Trait for the return type of [`#[root_task]`](crate::root_task) main functions.
pub trait Termination {
    type Error: fmt::Debug;

    fn report(self) -> Self::Error;
}

impl Termination for ! {
    type Error = !;

    fn report(self) -> Self::Error {
        self
    }
}

impl Termination for Never {
    type Error = Never;

    fn report(self) -> Self::Error {
        self
    }
}

impl<E: fmt::Debug> Termination for Result<!, E> {
    type Error = E;

    fn report(self) -> Self::Error {
        match self {
            #[allow(unreachable_patterns)]
            Ok(absurdity) => match absurdity {},
            Err(err) => err,
        }
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

// NOTE(rustc_wishlist) remove once #![never_type] is stabilized
/// Stable alternative to `!`.
///
/// This type in uninhabited like `!`, but does not require the unstable `#[feature(never_type)]`.
/// It implements [`Termination`], so it is useful in return types for
/// [`#[root_task]`](crate::root_task) main functions.
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Never {}

impl fmt::Display for Never {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        match *self {}
    }
}
