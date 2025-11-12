//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::cell::RefCell;
use core::ops::Deref;

use lock_api::{Mutex, RawMutex};

use crate::{WrappedMutex, WrappedRefCell, WrappedRefCellError};

pub use embedded_hal_nb::nb;
pub use embedded_hal_nb::serial::{Error, ErrorKind, ErrorType, Read, Write};

mod write_buffered;

pub use write_buffered::WriteBuffered;

// // //

impl<E: Error> Error for WrappedRefCellError<E> {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Contention => ErrorKind::Other,
            Self::Other(err) => err.kind(),
        }
    }
}

impl<T: Deref<Target = RefCell<U>>, U: ErrorType> ErrorType for &WrappedRefCell<T> {
    type Error = WrappedRefCellError<U::Error>;
}

impl<Word: Copy, T: Deref<Target = RefCell<U>>, U: Read<Word>> Read<Word> for &WrappedRefCell<T> {
    fn read(&mut self) -> nb::Result<Word, Self::Error> {
        self.try_borrow_mut()?
            .read()
            .map_err(|err| err.map(WrappedRefCellError::Other))
    }
}

impl<Word: Copy, T: Deref<Target = RefCell<U>>, U: Write<Word>> Write<Word> for &WrappedRefCell<T> {
    fn write(&mut self, word: Word) -> nb::Result<(), Self::Error> {
        self.try_borrow_mut()?
            .write(word)
            .map_err(|err| err.map(WrappedRefCellError::Other))
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.try_borrow_mut()?
            .flush()
            .map_err(|err| err.map(WrappedRefCellError::Other))
    }
}

impl<R: RawMutex, T: Deref<Target = Mutex<R, U>>, U: ErrorType> ErrorType for &WrappedMutex<T> {
    type Error = U::Error;
}

impl<R: RawMutex, Word: Copy, T: Deref<Target = Mutex<R, U>>, U: Read<Word>> Read<Word>
    for &WrappedMutex<T>
{
    fn read(&mut self) -> nb::Result<Word, Self::Error> {
        self.0.lock().read()
    }
}

impl<R: RawMutex, Word: Copy, T: Deref<Target = Mutex<R, U>>, U: Write<Word>> Write<Word>
    for &WrappedMutex<T>
{
    fn write(&mut self, word: Word) -> nb::Result<(), Self::Error> {
        self.0.lock().write(word)
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.0.lock().flush()
    }
}
