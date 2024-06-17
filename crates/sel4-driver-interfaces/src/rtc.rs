//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::cell::RefCell;
use core::ops::Deref;

use lock_api::{Mutex, RawMutex};
use rtcc::{DateTimeAccess, NaiveDateTime};

use crate::{WrappedMutex, WrappedRefCell, WrappedRefCellError};

impl<T: Deref<Target = RefCell<U>>, U: DateTimeAccess> DateTimeAccess for &WrappedRefCell<T> {
    type Error = WrappedRefCellError<U::Error>;

    fn datetime(&mut self) -> Result<NaiveDateTime, Self::Error> {
        self.with_mut(|this| this.datetime())
    }

    fn set_datetime(&mut self, datetime: &NaiveDateTime) -> Result<(), Self::Error> {
        self.with_mut(|this| this.set_datetime(datetime))
    }
}

impl<R: RawMutex, T: Deref<Target = Mutex<R, U>>, U: DateTimeAccess> DateTimeAccess
    for &WrappedMutex<T>
{
    type Error = U::Error;

    fn datetime(&mut self) -> Result<NaiveDateTime, Self::Error> {
        self.with_mut(|this| this.datetime())
    }

    fn set_datetime(&mut self, datetime: &NaiveDateTime) -> Result<(), Self::Error> {
        self.with_mut(|this| this.set_datetime(datetime))
    }
}
