//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::fmt;
use core::mem;

#[cfg(feature = "postcard")]
use serde::{Deserialize, Serialize};

use sel4_microkit_base::{with_msg_regs, with_msg_regs_mut, MessageInfo};

use sel4_microkit_message_types::{
    EmptyMessage, EmptyMessageValue, MessageLabel, MessageRecv, MessageRegisters,
    MessageRegistersMut, MessageRegistersPrefixLength, MessageSend, MessageValueRecv,
    MessageValueSend, TriviallyLabeled,
};

#[cfg(feature = "postcard")]
use sel4_microkit_message_types::MessageValueUsingPostcard;

pub use sel4_microkit_message_types as types;

const MAX_MESSAGE_LABEL: MessageLabel =
    !0 >> (mem::size_of::<MessageInfo>() * 8 - MessageInfo::label_width());

// // //

pub const UNSPECIFIED_ERROR_MESSAGE_LABEL: MessageLabel = MAX_MESSAGE_LABEL;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct UnspecifiedErrorMessage;

impl From<TriviallyLabeled<EmptyMessageValue, UNSPECIFIED_ERROR_MESSAGE_LABEL>>
    for UnspecifiedErrorMessage
{
    fn from(_: TriviallyLabeled<EmptyMessageValue, UNSPECIFIED_ERROR_MESSAGE_LABEL>) -> Self {
        Default::default()
    }
}

impl From<UnspecifiedErrorMessage>
    for TriviallyLabeled<EmptyMessageValue, UNSPECIFIED_ERROR_MESSAGE_LABEL>
{
    fn from(_: UnspecifiedErrorMessage) -> Self {
        Default::default()
    }
}

impl MessageSend for UnspecifiedErrorMessage {
    type Label = <TriviallyLabeled<EmptyMessageValue, UNSPECIFIED_ERROR_MESSAGE_LABEL> as MessageSend>::Label;

    type Error = <TriviallyLabeled<EmptyMessageValue, UNSPECIFIED_ERROR_MESSAGE_LABEL> as MessageSend>::Error;

    fn write_message(
        &self,
        regs: MessageRegistersMut,
    ) -> Result<(Self::Label, MessageRegistersPrefixLength), Self::Error> {
        <TriviallyLabeled<EmptyMessageValue, UNSPECIFIED_ERROR_MESSAGE_LABEL>>::from(*self)
            .write_message(regs)
    }
}

impl MessageRecv for UnspecifiedErrorMessage {
    type Label = <TriviallyLabeled<EmptyMessageValue, UNSPECIFIED_ERROR_MESSAGE_LABEL> as MessageRecv>::Label;

    type Error = <TriviallyLabeled<EmptyMessageValue, UNSPECIFIED_ERROR_MESSAGE_LABEL> as MessageRecv>::Error;

    fn read_message(label: Self::Label, regs: &MessageRegisters) -> Result<Self, Self::Error> {
        <TriviallyLabeled<EmptyMessageValue, UNSPECIFIED_ERROR_MESSAGE_LABEL>>::read_message(
            label, regs,
        )
        .map(Into::into)
    }
}

// // //

pub trait MessageInfoExt: Sized {
    fn send<T: MessageSend>(val: T) -> Result<Self, T::Error>;

    fn recv<T: MessageRecv>(self) -> Result<T, MessageRecvErrorFor<T>>;

    fn send_unspecified_error() -> Self {
        Self::send(UnspecifiedErrorMessage).unwrap_or_else(|absurdity| match absurdity {})
    }

    fn send_empty() -> Self {
        Self::send(EmptyMessage::new()).unwrap_or_else(|absurdity| match absurdity {})
    }

    fn recv_empty(self) -> Result<(), MessageRecvErrorFor<EmptyMessage>> {
        self.recv().map(|_: EmptyMessage| ())
    }

    fn send_with_trivial_label<T: MessageValueSend>(val: T) -> Result<Self, T::Error> {
        type Helper<T> = TriviallyLabeled<T>; // get default LABEL
        Self::send(Helper::new(val))
    }

    fn recv_with_trivial_label<T: MessageValueRecv>(
        self,
    ) -> Result<T, MessageRecvErrorFor<TriviallyLabeled<T>>> {
        self.recv().map(TriviallyLabeled::into_inner)
    }

    #[cfg(feature = "postcard")]
    fn send_using_postcard<T: Serialize>(
        val: T,
    ) -> Result<Self, <MessageValueUsingPostcard<T> as MessageValueSend>::Error> {
        Self::send_with_trivial_label(MessageValueUsingPostcard(val))
    }

    #[cfg(feature = "postcard")]
    fn recv_using_postcard<T: for<'a> Deserialize<'a>>(
        self,
    ) -> Result<T, MessageRecvErrorFor<TriviallyLabeled<MessageValueUsingPostcard<T>>>> {
        self.recv_with_trivial_label()
            .map(|MessageValueUsingPostcard(val)| val)
    }
}

impl MessageInfoExt for MessageInfo {
    fn send<T: MessageSend>(val: T) -> Result<Self, T::Error> {
        let (label, prefix_len) =
            with_msg_regs_mut(|buf| val.write_message(MessageRegistersMut::new(buf)))?;
        let label = label.into();
        assert!(label <= MAX_MESSAGE_LABEL);
        // assert!(label != UNSPECIFIED_ERROR_MESSAGE_LABEL);
        Ok(Self::new(label, prefix_len.into_inner()))
    }

    fn recv<T: MessageRecv>(self) -> Result<T, MessageRecvErrorFor<T>> {
        // if self.label() == UNSPECIFIED_ERROR_MESSAGE_LABEL) {
        //     return Err(MessageRecvError::Unspecified);
        // }
        let label = self
            .label()
            .try_into()
            .map_err(MessageRecvError::LabelError)?;
        with_msg_regs(|buf| T::read_message(label, &MessageRegisters::new(&buf[..self.count()])))
            .map_err(MessageRecvError::ValueError)
    }

    // fn send_unspecified_error() -> Self {
    //     Self::new(UNSPECIFIED_ERROR_MESSAGE_LABEL, 0)
    // }
}

pub type MessageRecvErrorFor<T> = MessageRecvError<
    <<T as MessageRecv>::Label as TryFrom<MessageLabel>>::Error,
    <T as MessageRecv>::Error,
>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MessageRecvError<E1, E2> {
    LabelError(E1),
    ValueError(E2),
    // Unspecified,
}

impl<E1: fmt::Display, E2: fmt::Display> fmt::Display for MessageRecvError<E1, E2> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::LabelError(err) => write!(f, "label error: {}", err),
            Self::ValueError(err) => write!(f, "value error: {}", err),
            // Self::Unspecified => write!(f, "unspecified error"),
        }
    }
}
