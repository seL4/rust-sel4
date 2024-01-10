//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;

pub trait Termination {
    type Error;

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

impl<E> Termination for Result<!, E> {
    type Error = E;

    fn report(self) -> Self::Error {
        match self {
            Ok(absurdity) => match absurdity {},
            Err(err) => err,
        }
    }
}

impl<E> Termination for Result<Never, E> {
    type Error = E;

    fn report(self) -> Self::Error {
        match self {
            Ok(absurdity) => match absurdity {},
            Err(err) => err,
        }
    }
}

// NOTE(rustc_wishlist) remove once #![never_type] is stabilized
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Never {}

impl fmt::Display for Never {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        match *self {}
    }
}
