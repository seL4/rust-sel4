#![no_std]
#![feature(int_roundings)]

use core::fmt;
use core::mem;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use zerocopy::{AsBytes, FromBytes, Unalign};

use sel4cp::message::*;

pub trait MessageInfoExt {
    fn _msg_info(&self) -> &MessageInfo;

    fn send<T: AsBytes>(label: impl Into<MessageLabel>, val: T) -> MessageInfo {
        Self::try_send(label, val).unwrap()
    }

    fn try_send<T: AsBytes>(
        label: impl Into<MessageLabel>,
        val: T,
    ) -> Result<MessageInfo, MessageInfoSendError> {
        with_msg_bytes_mut(|bytes| {
            val.write_to_prefix(bytes)
                .ok_or(MessageInfoSendError::ValueTooLarge)
        })?;
        Ok(MessageInfo::new(
            label.into(),
            bytes_to_mrs(mem::size_of_val(&val)),
        ))
    }

    fn recv<T: FromBytes + Copy>(&self) -> Result<T, MessageInfoRecvError> {
        with_msg_bytes(|bytes| -> Result<T, MessageInfoRecvError> {
            Unalign::<T>::read_from_prefix(&bytes[..mrs_to_bytes(self._msg_info().count())])
                .ok_or(MessageInfoRecvError::MessageTooShort)
                .map(|unalign| unalign.get())
        })
    }
}

impl MessageInfoExt for MessageInfo {
    fn _msg_info(&self) -> &MessageInfo {
        self
    }
}

fn mrs_to_bytes(num_mrs: usize) -> usize {
    num_mrs * mem::size_of::<MessageRegisterValue>()
}

fn bytes_to_mrs(num_bytes: usize) -> usize {
    let d = mem::size_of::<MessageRegisterValue>();
    num_bytes.next_multiple_of(d) / d
}

#[derive(Debug, Clone)]
pub enum MessageInfoSendError {
    ValueTooLarge,
}

#[derive(Debug, Clone)]
pub enum MessageInfoRecvError {
    MessageTooShort,
}

// // //

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct NoMessageLabel;

impl From<NoMessageLabel> for MessageLabel {
    fn from(_: NoMessageLabel) -> Self {
        Self::default()
    }
}

impl TryFrom<MessageLabel> for NoMessageLabel {
    type Error = TryFromNoMessageLabelError;

    fn try_from(val: MessageLabel) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Self),
            _ => Err(TryFromNoMessageLabelError(())),
        }
    }
}

pub struct TryFromNoMessageLabelError(());

impl fmt::Display for TryFromNoMessageLabelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unexpected label value for NoMessageLabel")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(target_pointer_width = "32", repr(u32))]
#[cfg_attr(target_pointer_width = "64", repr(u64))]
pub enum StatusMessageLabel {
    Ok,
    Error,
}

impl StatusMessageLabel {
    pub fn is_ok(&self) -> bool {
        *self == StatusMessageLabel::Ok
    }

    pub fn is_error(&self) -> bool {
        *self == StatusMessageLabel::Error
    }
}

// // //

#[derive(Clone, Copy, PartialEq, Eq, AsBytes, FromBytes)]
#[repr(C)]
pub struct NoMessageValue;
