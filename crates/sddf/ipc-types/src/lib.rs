//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::convert::Infallible;
use core::error::Error;
use core::fmt;

// // //

mod empty_message;
mod message_builder;
mod message_parser;

pub use empty_message::EmptyMessage;
pub use message_builder::{IntoMessageRegisterValue, MessageBuilder};
pub use message_parser::{MessagParseError, MessageParser, TryFromMessageRegisterValue};

#[cfg(feature = "sel4-microkit-base")]
mod when_microkit;

// // //

#[cfg(target_pointer_width = "32")]
type Word = u32;

#[cfg(target_pointer_width = "64")]
type Word = u64;

// // //

#[allow(dead_code)]
fn bytes_to_words(num_bytes: usize) -> usize {
    let d = size_of::<MessageRegisterValue>();
    num_bytes.next_multiple_of(d) / d
}

// // //

pub type MessageLabel = Word;

pub type MessageRegisterValue = Word;

pub trait MessageReader<T> {
    type Error: Error;

    fn read_message(
        &self,
        label: MessageLabel,
        buf: &[MessageRegisterValue],
    ) -> Result<T, Self::Error>;
}

pub trait MessageWriter {
    type Error: Error;

    fn write_message(
        &self,
        buf: &mut [MessageRegisterValue],
    ) -> Result<(MessageLabel, usize), Self::Error>;
}

impl<E: Error, T, F: Fn(MessageLabel, &[MessageRegisterValue]) -> Result<T, E>> MessageReader<T>
    for F
{
    type Error = E;

    fn read_message(
        &self,
        label: MessageLabel,
        buf: &[MessageRegisterValue],
    ) -> Result<T, Self::Error> {
        (self)(label, buf)
    }
}

impl<E: Error, F: Fn(&mut [MessageRegisterValue]) -> Result<(MessageLabel, usize), E>> MessageWriter
    for F
{
    type Error = E;

    fn write_message(
        &self,
        buf: &mut [MessageRegisterValue],
    ) -> Result<(MessageLabel, usize), Self::Error> {
        (self)(buf)
    }
}

// // //

pub trait ReadFromMessage: Sized {
    type Error: Error;

    fn read_from_message(
        label: MessageLabel,
        buf: &[MessageRegisterValue],
    ) -> Result<Self, Self::Error>;
}

#[derive(Default)]
pub struct ImplicitMessageReader;

impl ImplicitMessageReader {
    pub const fn new() -> Self {
        Self
    }
}

impl<T: ReadFromMessage> MessageReader<T> for ImplicitMessageReader {
    type Error = T::Error;

    fn read_message(
        &self,
        label: MessageLabel,
        buf: &[MessageRegisterValue],
    ) -> Result<T, Self::Error> {
        T::read_from_message(label, buf)
    }
}

// // //

pub trait CallTarget {
    fn call<T, W: MessageWriter, R: MessageReader<T>>(
        &self,
        writer: W,
        reader: R,
    ) -> Result<T, CallError<W::Error, R::Error>>;

    fn call_with_implicit_reader<T: ReadFromMessage, W: MessageWriter>(
        &self,
        writer: W,
    ) -> Result<T, CallError<W::Error, T::Error>> {
        self.call(writer, ImplicitMessageReader::new())
    }
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum CallError<W, R> {
    WriteError(W),
    ReadError(R),
}

impl<R> CallError<Infallible, R> {
    pub fn into_reader_error(self) -> R {
        match self {
            Self::ReadError(err) => err,
        }
    }
}

impl<W: fmt::Display, R: fmt::Display> fmt::Display for CallError<W, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::WriteError(err) => write!(f, "write error: {}", err),
            Self::ReadError(err) => write!(f, "read error: {}", err),
        }
    }
}

impl<W: fmt::Debug + fmt::Display, R: fmt::Debug + fmt::Display> Error for CallError<W, R> {}
