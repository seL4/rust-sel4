//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

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

impl<E> Termination for Result<!, E> {
    type Error = E;

    fn report(self) -> Self::Error {
        self.into_err()
    }
}
