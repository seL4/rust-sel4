#![no_std]
#![feature(int_roundings)]

use core::mem;

use serde::{Deserialize, Serialize};

use sel4cp::message::{
    with_msg_bytes, with_msg_bytes_mut, MessageInfo, MessageLabel, MessageRegisterValue,
};

pub fn send<T: Serialize>(label: impl Into<MessageLabel>, val: T) -> MessageInfo {
    try_send(label, val).unwrap()
}

pub fn try_send<T: Serialize>(
    label: impl Into<MessageLabel>,
    val: T,
) -> Result<MessageInfo, postcard::Error> {
    let count = with_msg_bytes_mut(|bytes| {
        postcard::to_slice(&val, bytes).map(|used| {
            used.len()
                .next_multiple_of(mem::size_of::<MessageRegisterValue>())
                / mem::size_of::<MessageRegisterValue>()
        })
    })?;
    Ok(MessageInfo::new(label.into(), count))
}

pub fn recv<T: for<'a> Deserialize<'a>>(msg_info: &MessageInfo) -> Result<T, postcard::Error> {
    with_msg_bytes(|bytes| -> Result<T, postcard::Error> {
        let num_bytes = msg_info.count() * mem::size_of::<MessageRegisterValue>();
        postcard::from_bytes(&bytes[..num_bytes])
    })
}

pub trait MessageInfoExt {
    fn _msg_info(&self) -> &MessageInfo;

    fn send_with_postcard<T: Serialize>(label: impl Into<MessageLabel>, val: T) -> MessageInfo {
        send(label, val)
    }

    fn try_send_with_postcard<T: Serialize>(
        label: impl Into<MessageLabel>,
        val: T,
    ) -> Result<MessageInfo, postcard::Error> {
        try_send(label, val)
    }

    fn recv_with_postcard<T: for<'a> Deserialize<'a>>(&self) -> Result<T, postcard::Error> {
        recv(self._msg_info())
    }
}

impl MessageInfoExt for MessageInfo {
    fn _msg_info(&self) -> &MessageInfo {
        self
    }
}
