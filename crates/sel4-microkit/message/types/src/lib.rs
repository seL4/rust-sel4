//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::convert::Infallible;
use core::fmt;
use core::mem;

use zerocopy::{FromBytes, Immutable, IntoBytes, Unalign};

#[cfg(feature = "postcard")]
mod when_postcard;

#[cfg(feature = "postcard")]
pub use when_postcard::MessageValueUsingPostcard;

#[cfg(target_pointer_width = "32")]
type Word = u32;

#[cfg(target_pointer_width = "64")]
type Word = u64;

pub type MessageLabel = Word;

pub type MessageRegisterValue = Word;

pub trait MessageValueSend {
    type Error;

    fn write_message_value(&self, buf: &mut [u8]) -> Result<usize, Self::Error>;
}

pub trait MessageValueRecv: Sized {
    type Error;

    fn read_message_value(buf: &[u8]) -> Result<Self, Self::Error>;
}

pub trait MessageSend {
    type Label: Into<MessageLabel>;

    type Error;

    fn write_message(&self, buf: &mut [u8]) -> Result<(Self::Label, usize), Self::Error>;
}

pub trait MessageRecv: Sized {
    type Label: TryFrom<MessageLabel>;

    type Error;

    fn read_message(label: Self::Label, buf: &[u8]) -> Result<Self, Self::Error>;
}

// // //

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct EmptyMessageValue;

impl MessageValueSend for EmptyMessageValue {
    type Error = Infallible;

    fn write_message_value(&self, _buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

impl MessageValueRecv for EmptyMessageValue {
    type Error = RecvEmptyMessageValueError;

    fn read_message_value(buf: &[u8]) -> Result<Self, Self::Error> {
        if buf.is_empty() {
            Ok(Self)
        } else {
            Err(Self::Error::MessageIsNotEmpty)
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum RecvEmptyMessageValueError {
    MessageIsNotEmpty,
}

// // //

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ConstMessageLabel<const LABEL: MessageLabel>(());

impl<const LABEL: MessageLabel> ConstMessageLabel<LABEL> {
    pub const fn new() -> Self {
        Self(())
    }
}

impl<const LABEL: MessageLabel> From<ConstMessageLabel<LABEL>> for MessageLabel {
    fn from(_: ConstMessageLabel<LABEL>) -> Self {
        LABEL
    }
}

impl<const LABEL: MessageLabel> TryFrom<MessageLabel> for ConstMessageLabel<LABEL> {
    type Error = TryFromConstMessageLabelError<LABEL>;

    fn try_from(val: MessageLabel) -> Result<Self, Self::Error> {
        if val == LABEL {
            Ok(Self::new())
        } else {
            Err(TryFromConstMessageLabelError(()))
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TryFromConstMessageLabelError<const LABEL: MessageLabel>(());

impl<const LABEL: MessageLabel> fmt::Display for TryFromConstMessageLabelError<LABEL> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "expected label value of {}", LABEL)
    }
}

// // //

pub const DEFAULT_TRIVIAL_MESSAGE_LABEL: MessageLabel = 0;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TriviallyLabeled<T, const LABEL: MessageLabel = DEFAULT_TRIVIAL_MESSAGE_LABEL>(T);

impl<T, const LABEL: MessageLabel> TriviallyLabeled<T, LABEL> {
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: MessageValueSend, const LABEL: MessageLabel> MessageSend for TriviallyLabeled<T, LABEL> {
    type Label = ConstMessageLabel<LABEL>;

    type Error = <T as MessageValueSend>::Error;

    fn write_message(&self, buf: &mut [u8]) -> Result<(Self::Label, usize), Self::Error> {
        Ok((ConstMessageLabel::new(), self.0.write_message_value(buf)?))
    }
}

impl<T: MessageValueRecv, const LABEL: MessageLabel> MessageRecv for TriviallyLabeled<T, LABEL> {
    type Label = ConstMessageLabel<LABEL>;

    type Error = <T as MessageValueRecv>::Error;

    fn read_message(_: Self::Label, buf: &[u8]) -> Result<Self, Self::Error> {
        T::read_message_value(buf).map(TriviallyLabeled::new)
    }
}

// // //

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct EmptyMessage;

impl From<TriviallyLabeled<EmptyMessageValue>> for EmptyMessage {
    fn from(_: TriviallyLabeled<EmptyMessageValue>) -> Self {
        Default::default()
    }
}

impl From<EmptyMessage> for TriviallyLabeled<EmptyMessageValue> {
    fn from(_: EmptyMessage) -> Self {
        Default::default()
    }
}

impl MessageSend for EmptyMessage {
    type Label = <TriviallyLabeled<EmptyMessageValue> as MessageSend>::Label;

    type Error = <TriviallyLabeled<EmptyMessageValue> as MessageSend>::Error;

    fn write_message(&self, buf: &mut [u8]) -> Result<(Self::Label, usize), Self::Error> {
        <TriviallyLabeled<EmptyMessageValue>>::from(*self).write_message(buf)
    }
}

impl MessageRecv for EmptyMessage {
    type Label = <TriviallyLabeled<EmptyMessageValue> as MessageRecv>::Label;

    type Error = <TriviallyLabeled<EmptyMessageValue> as MessageRecv>::Error;

    fn read_message(label: Self::Label, buf: &[u8]) -> Result<Self, Self::Error> {
        <TriviallyLabeled<EmptyMessageValue>>::read_message(label, buf).map(Into::into)
    }
}

// // //

impl<T: IntoBytes + Immutable> MessageValueSend for T {
    type Error = SendIntoBytesError;

    fn write_message_value(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.write_to_prefix(buf)
            .map_err(|_| SendIntoBytesError::ValueTooLarge)?;
        Ok(mem::size_of_val(&self))
    }
}

impl<T: FromBytes + Copy> MessageValueRecv for T {
    type Error = RecvFromBytesError;

    fn read_message_value(buf: &[u8]) -> Result<Self, Self::Error> {
        Ok(Unalign::<T>::read_from_prefix(buf)
            .map_err(|_| RecvFromBytesError::MessageTooShort)?
            .0
            .get())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SendIntoBytesError {
    ValueTooLarge,
}

#[derive(Copy, Clone, Debug)]
pub enum RecvFromBytesError {
    MessageTooShort,
}
