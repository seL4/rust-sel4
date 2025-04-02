//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::error::Error;
use core::fmt;
use core::marker::PhantomData;
use core::mem;

use zerocopy::{FromBytes, Immutable, IntoBytes, Unalign};

use crate::{bytes_to_words, MessageLabel, MessageReader, MessageRegisterValue, MessageWriter};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ZerocopyMessageReader;

impl ZerocopyMessageReader {
    pub const fn new() -> Self {
        Self
    }
}

impl<T: FromBytes + Copy> MessageReader<T> for ZerocopyMessageReader {
    type Error = ZerocopyMessageReaderError;

    fn read_message(
        &self,
        label: MessageLabel,
        buf: &[MessageRegisterValue],
    ) -> Result<T, Self::Error> {
        if label != 0 {
            return Err(ZerocopyMessageReaderError::UnexpectedLabel { label });
        }
        Ok(Unalign::<T>::read_from_prefix(buf.as_bytes())
            .map_err(|_| ZerocopyMessageReaderError::MessageTooShort)?
            .0
            .get())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ZerocopyMessageWriter<T>(T);

impl<T> ZerocopyMessageWriter<T> {
    pub const fn new(inner: T) -> Self {
        Self(inner)
    }
}

impl<T: IntoBytes + Immutable> MessageWriter for ZerocopyMessageWriter<T> {
    type Error = ZerocopyMessageWriterError;

    fn write_message(
        &self,
        buf: &mut [MessageRegisterValue],
    ) -> Result<(MessageLabel, usize), Self::Error> {
        self.0
            .write_to_prefix(buf.as_mut_bytes())
            .map_err(|_| ZerocopyMessageWriterError::ValueTooLarge)?;
        Ok((0, bytes_to_words(mem::size_of_val(self))))
    }
}

#[derive(Clone, Debug)]
pub enum ZerocopyMessageReaderError {
    UnexpectedLabel { label: MessageLabel },
    MessageTooShort,
}

impl fmt::Display for ZerocopyMessageReaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedLabel { label } => write!(f, "unexpected label: {label}"),
            Self::MessageTooShort => write!(f, "message too short"),
        }
    }
}

impl Error for ZerocopyMessageReaderError {}

#[derive(Clone, Debug)]
pub enum ZerocopyMessageWriterError {
    ValueTooLarge,
}

impl fmt::Display for ZerocopyMessageWriterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ValueTooLarge => write!(f, "value too large"),
        }
    }
}

impl Error for ZerocopyMessageWriterError {}
