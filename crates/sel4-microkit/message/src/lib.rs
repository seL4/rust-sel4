#![no_std]
#![feature(int_roundings)]
#![feature(never_type)]
#![feature(unwrap_infallible)]

use core::fmt;
use core::mem;

#[cfg(feature = "postcard")]
use serde::{Deserialize, Serialize};

use sel4_microkit::{with_msg_bytes, with_msg_bytes_mut, MessageInfo, MessageRegisterValue};

use sel4_microkit_message_types::{
    EmptyMessage, MessageLabel, MessageRecv, MessageSend, MessageValueRecv, MessageValueSend,
    TriviallyLabeled, TryFromDefaultMessageLabelError,
};

#[cfg(feature = "postcard")]
use sel4_microkit_message_types::MessageValueUsingPostcard;

pub use sel4_microkit_message_types as types;

pub const UNSPECIFIED_ERROR_LABEL: MessageLabel = (1 << MessageInfo::label_width()) - 1;

pub trait MessageInfoExt: Sized {
    fn send<T: MessageSend>(val: T) -> Result<Self, T::Error>;

    fn recv<T: MessageRecv>(self) -> Result<T, MessageRecvErrorFor<T>>;

    fn send_unspecified_error() -> Self;

    fn send_empty() -> Self {
        Self::send(EmptyMessage).into_ok()
    }

    fn recv_empty(self) -> Result<(), MessageRecvErrorFor<EmptyMessage>> {
        self.recv().map(|EmptyMessage| ())
    }

    fn send_with_trivial_label<T: MessageValueSend>(val: T) -> Result<Self, T::Error> {
        Self::send(TriviallyLabeled(val))
    }

    fn recv_with_trivial_label<T: MessageValueRecv>(
        self,
    ) -> Result<T, MessageRecvErrorFor<TriviallyLabeled<T>>> {
        self.recv().map(|TriviallyLabeled(val)| val)
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
    ) -> Result<
        T,
        MessageRecvError<
            TryFromDefaultMessageLabelError,
            <MessageValueUsingPostcard<T> as MessageValueRecv>::Error,
        >,
    > {
        self.recv_with_trivial_label()
            .map(|MessageValueUsingPostcard(val)| val)
    }
}

impl MessageInfoExt for MessageInfo {
    fn send<T: MessageSend>(val: T) -> Result<Self, T::Error> {
        let (label, num_bytes) = with_msg_bytes_mut(|buf| val.write_message(buf))?;
        let label = label.into();
        assert!(label < UNSPECIFIED_ERROR_LABEL); // TODO return error instead?
        Ok(Self::new(label, bytes_to_mrs(num_bytes)))
    }

    fn recv<T: MessageRecv>(self) -> Result<T, MessageRecvErrorFor<T>> {
        if self.label() >= UNSPECIFIED_ERROR_LABEL {
            return Err(MessageRecvError::Unspecified);
        }
        let label = self
            .label()
            .try_into()
            .map_err(MessageRecvError::LabelError)?;
        with_msg_bytes(|buf| T::read_message(label, &buf[..mrs_to_bytes(self.count())]))
            .map_err(MessageRecvError::ValueError)
    }

    fn send_unspecified_error() -> Self {
        Self::new(UNSPECIFIED_ERROR_LABEL, 0)
    }
}

pub type MessageRecvErrorFor<T> = MessageRecvError<
    <<T as MessageRecv>::Label as TryFrom<MessageLabel>>::Error,
    <T as MessageRecv>::Error,
>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MessageRecvError<E1, E2> {
    LabelError(E1),
    ValueError(E2),
    Unspecified,
}

impl<E1: fmt::Display, E2: fmt::Display> fmt::Display for MessageRecvError<E1, E2> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::LabelError(err) => write!(f, "label error: {}", err),
            Self::ValueError(err) => write!(f, "value error: {}", err),
            Self::Unspecified => write!(f, "unspecified error"),
        }
    }
}

fn mrs_to_bytes(num_mrs: usize) -> usize {
    num_mrs * mem::size_of::<MessageRegisterValue>()
}

fn bytes_to_mrs(num_bytes: usize) -> usize {
    let d = mem::size_of::<MessageRegisterValue>();
    num_bytes.next_multiple_of(d) / d
}
