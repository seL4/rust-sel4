use core::fmt;
use core::mem;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use zerocopy::{AsBytes, FromBytes, Unalign};

#[derive(Debug)]
pub struct MessageInfo {
    inner: sel4::MessageInfo,
}

impl MessageInfo {
    pub(crate) fn from_sel4(inner: sel4::MessageInfo) -> Self {
        Self { inner }
    }

    pub(crate) fn into_sel4(self) -> sel4::MessageInfo {
        self.inner
    }

    pub fn new(label: MessageLabel, count: usize) -> Self {
        Self::from_sel4(sel4::MessageInfo::new(label, 0, 0, count))
    }

    pub fn label(&self) -> MessageLabel {
        self.inner.label()
    }

    pub fn count(&self) -> usize {
        self.inner.length()
    }

    pub fn send<T: AsBytes>(label: impl Into<MessageLabel>, val: T) -> Self {
        Self::try_send(label, val).unwrap()
    }

    pub fn try_send<T: AsBytes>(
        label: impl Into<MessageLabel>,
        val: T,
    ) -> Result<Self, MessageInfoSendError> {
        let count = mem::size_of_val(&val).next_multiple_of(mem::size_of::<MessageRegisterValue>())
            / mem::size_of::<MessageRegisterValue>();
        with_msg_bytes_mut(|bytes| {
            val.write_to_prefix(bytes)
                .ok_or(MessageInfoSendError::ValueTooLarge)
        })?;
        Ok(Self::new(label.into(), count))
    }

    pub fn recv<T: FromBytes + Copy>(&self) -> Result<T, MessageInfoRecvError> {
        with_msg_bytes(|bytes| -> Result<T, MessageInfoRecvError> {
            let num_bytes = self.count() * mem::size_of::<MessageRegisterValue>();
            Unalign::read_from_prefix(&bytes[..num_bytes])
                .ok_or(MessageInfoRecvError::MessageTooShort)
                .map(|unalign| unalign.get())
        })
    }
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

pub type MessageLabel = sel4::Word;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct NoMessageLabel;

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

// // //

pub type MessageRegisterValue = sel4::Word;

#[derive(Clone, Copy, PartialEq, Eq, AsBytes, FromBytes)]
#[repr(C)]
pub struct NoMessageValue;

// // //

pub fn with_msg_regs<T>(f: impl FnOnce(&[MessageRegisterValue]) -> T) -> T {
    sel4::with_borrow_ipc_buffer(|ipc_buffer| f(ipc_buffer.msg_regs()))
}

pub fn with_msg_regs_mut<T>(f: impl FnOnce(&mut [MessageRegisterValue]) -> T) -> T {
    sel4::with_borrow_ipc_buffer_mut(|ipc_buffer| f(ipc_buffer.msg_regs_mut()))
}

pub fn with_msg_bytes<T>(f: impl FnOnce(&[u8]) -> T) -> T {
    sel4::with_borrow_ipc_buffer(|ipc_buffer| f(ipc_buffer.msg_bytes()))
}

pub fn with_msg_bytes_mut<T>(f: impl FnOnce(&mut [u8]) -> T) -> T {
    sel4::with_borrow_ipc_buffer_mut(|ipc_buffer| f(ipc_buffer.msg_bytes_mut()))
}

pub fn set_mr(i: usize, value: MessageRegisterValue) {
    with_msg_regs_mut(|regs| regs[i] = value)
}

pub fn get_mr(i: usize) -> MessageRegisterValue {
    with_msg_regs(|regs| regs[i])
}
