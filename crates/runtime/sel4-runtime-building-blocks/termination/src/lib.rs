#![no_std]
#![feature(never_type)]

use core::convert::Infallible;
use core::fmt;
use core::write;

pub trait Termination {
    fn report(self, writer: impl fmt::Write) -> fmt::Result;
}

impl Termination for () {
    fn report(self, _writer: impl fmt::Write) -> fmt::Result {
        Ok(())
    }
}

impl Termination for ! {
    fn report(self, _writer: impl fmt::Write) -> fmt::Result {
        self
    }
}

impl Termination for Infallible {
    fn report(self, _writer: impl fmt::Write) -> fmt::Result {
        match self {}
    }
}

impl<T: Termination, E: fmt::Debug> Termination for Result<T, E> {
    fn report(self, mut writer: impl fmt::Write) -> fmt::Result {
        match self {
            Ok(val) => val.report(writer),
            Err(err) => write!(writer, "Error: {err:?}"),
        }
    }
}
