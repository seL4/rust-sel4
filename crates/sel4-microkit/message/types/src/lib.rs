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

pub struct MessageRegisters<'a> {
    registers: &'a [MessageRegisterValue],
}

pub struct MessageRegistersMut<'a> {
    registers: &'a mut [MessageRegisterValue],
}

pub struct MessageRegistersPrefixLength(usize);

impl MessageRegistersPrefixLength {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn into_inner(self) -> usize {
        self.0
    }
}

impl<'a> MessageRegisters<'a> {
    pub fn new(registers: &'a [MessageRegisterValue]) -> Self {
        Self { registers }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.registers.as_bytes()
    }

    pub fn as_words(&self) -> &[MessageRegisterValue] {
        &self.registers
    }
}

impl<'a> MessageRegistersMut<'a> {
    pub fn new(registers: &'a mut [MessageRegisterValue]) -> Self {
        Self { registers }
    }

    pub fn with_bytes<E>(
        self,
        f: impl FnOnce(&mut [u8]) -> Result<usize, E>,
    ) -> Result<MessageRegistersPrefixLength, E> {
        Ok(MessageRegistersPrefixLength(bytes_to_mrs(f(self
            .registers
            .as_mut_bytes())?)))
    }

    pub fn with_words<E>(
        self,
        f: impl FnOnce(&mut [MessageRegisterValue]) -> Result<usize, E>,
    ) -> Result<MessageRegistersPrefixLength, E> {
        Ok(MessageRegistersPrefixLength(f(self.registers)?))
    }
}

pub trait MessageValueSend {
    type Error;

    fn write_message_value(
        &self,
        regs: MessageRegistersMut,
    ) -> Result<MessageRegistersPrefixLength, Self::Error>;
}

pub trait MessageValueRecv: Sized {
    type Error;

    fn read_message_value(regs: &MessageRegisters) -> Result<Self, Self::Error>;
}

pub trait MessageSend {
    type Label: Into<MessageLabel>;

    type Error;

    fn write_message(
        &self,
        regs: MessageRegistersMut,
    ) -> Result<(Self::Label, MessageRegistersPrefixLength), Self::Error>;
}

pub trait MessageRecv: Sized {
    type Label: TryFrom<MessageLabel>;

    type Error;

    fn read_message(label: Self::Label, regs: &MessageRegisters) -> Result<Self, Self::Error>;
}

// // //

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct EmptyMessageValue(());

impl EmptyMessageValue {
    pub const fn new() -> Self {
        Self(())
    }
}

impl MessageValueSend for EmptyMessageValue {
    type Error = Infallible;

    fn write_message_value(
        &self,
        _regs: MessageRegistersMut,
    ) -> Result<MessageRegistersPrefixLength, Self::Error> {
        Ok(MessageRegistersPrefixLength::empty())
    }
}

impl MessageValueRecv for EmptyMessageValue {
    type Error = RecvEmptyMessageValueError;

    fn read_message_value(regs: &MessageRegisters) -> Result<Self, Self::Error> {
        if regs.as_bytes().is_empty() {
            Ok(Self::new())
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TriviallyLabeled<T, const LABEL: MessageLabel = 0>(T);

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

    fn write_message(
        &self,
        regs: MessageRegistersMut,
    ) -> Result<(Self::Label, MessageRegistersPrefixLength), Self::Error> {
        Ok((ConstMessageLabel::new(), self.0.write_message_value(regs)?))
    }
}

impl<T: MessageValueRecv, const LABEL: MessageLabel> MessageRecv for TriviallyLabeled<T, LABEL> {
    type Label = ConstMessageLabel<LABEL>;

    type Error = <T as MessageValueRecv>::Error;

    fn read_message(_: Self::Label, regs: &MessageRegisters) -> Result<Self, Self::Error> {
        T::read_message_value(regs).map(TriviallyLabeled::new)
    }
}

// // //

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct EmptyMessage(());

impl EmptyMessage {
    pub const fn new() -> Self {
        Self(())
    }
}

impl MessageSend for EmptyMessage {
    type Label = <TriviallyLabeled<EmptyMessageValue> as MessageSend>::Label;

    type Error = <TriviallyLabeled<EmptyMessageValue> as MessageSend>::Error;

    fn write_message(
        &self,
        regs: MessageRegistersMut,
    ) -> Result<(Self::Label, MessageRegistersPrefixLength), Self::Error> {
        <TriviallyLabeled<EmptyMessageValue>>::default().write_message(regs)
    }
}

impl MessageRecv for EmptyMessage {
    type Label = <TriviallyLabeled<EmptyMessageValue> as MessageRecv>::Label;

    type Error = <TriviallyLabeled<EmptyMessageValue> as MessageRecv>::Error;

    fn read_message(label: Self::Label, regs: &MessageRegisters) -> Result<Self, Self::Error> {
        <TriviallyLabeled<EmptyMessageValue>>::read_message(label, regs).map(|_| Default::default())
    }
}

// // //

impl<T: IntoBytes + Immutable> MessageValueSend for T {
    type Error = SendIntoBytesError;

    fn write_message_value(
        &self,
        regs: MessageRegistersMut,
    ) -> Result<MessageRegistersPrefixLength, Self::Error> {
        regs.with_bytes(|buf| {
            self.write_to_prefix(buf)
                .map_err(|_| SendIntoBytesError::ValueTooLarge)?;
            Ok(mem::size_of_val(&self))
        })
    }
}

impl<T: FromBytes + Copy> MessageValueRecv for T {
    type Error = RecvFromBytesError;

    fn read_message_value(regs: &MessageRegisters) -> Result<Self, Self::Error> {
        Ok(Unalign::<T>::read_from_prefix(regs.as_bytes())
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

// // //

fn bytes_to_mrs(num_bytes: usize) -> usize {
    let d = mem::size_of::<MessageRegisterValue>();
    num_bytes.next_multiple_of(d) / d
}
