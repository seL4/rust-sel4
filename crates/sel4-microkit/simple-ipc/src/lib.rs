//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::error::Error;
use core::fmt;

use serde::{Deserialize, Serialize};
use zerocopy::IntoBytes;

use sel4_microkit_base::{
    Channel, MessageInfo, MessageLabel, MessageRegisterValue, with_msg_regs, with_msg_regs_mut,
};

const MAX_MESSAGE_LABEL: MessageLabel =
    !0 >> (size_of::<MessageInfo>() * 8 - MessageInfo::label_width());

pub const UNSPECIFIED_ERROR_MESSAGE_LABEL: MessageLabel = MAX_MESSAGE_LABEL;

pub fn send_unspecified_error() -> MessageInfo {
    MessageInfo::new(UNSPECIFIED_ERROR_MESSAGE_LABEL, 0)
}

pub fn try_send<T: Serialize>(val: T) -> Result<MessageInfo, postcard::Error> {
    with_msg_regs_mut(|buf| {
        let used = postcard::to_slice(&val, buf.as_mut_bytes())?;
        let count = bytes_to_words(used.len());
        Ok(MessageInfo::new(0, count))
    })
}

pub fn send<T: Serialize>(val: T) -> MessageInfo {
    try_send(val).unwrap()
}

pub fn recv<T: for<'a> Deserialize<'a>>(msg_info: MessageInfo) -> Result<T, RecvError> {
    let label = msg_info.label();
    if label != 0 {
        return Err(RecvError::UnexpectedLabel { label });
    }
    with_msg_regs(|buf| postcard::from_bytes(buf.as_bytes()).map_err(RecvError::PostcardError))
}

pub fn try_call<T: Serialize, U: for<'a> Deserialize<'a>>(
    channel: Channel,
    val: T,
) -> Result<U, TryCallError> {
    let req_msg_info = try_send(val).map_err(TryCallError::SendError)?;
    let resp_msg_info = channel.pp_call(req_msg_info);
    recv(resp_msg_info).map_err(TryCallError::RecvError)
}

pub fn call<T: Serialize, U: for<'a> Deserialize<'a>>(
    channel: Channel,
    val: T,
) -> Result<U, RecvError> {
    try_call(channel, val).map_err(|err| match err {
        TryCallError::SendError(err) => panic!("send error: {err}"),
        TryCallError::RecvError(err) => err,
    })
}

#[derive(Clone, Debug)]
pub enum RecvError {
    UnexpectedLabel { label: MessageLabel },
    PostcardError(postcard::Error),
}

impl fmt::Display for RecvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedLabel { label } => write!(f, "unexpected label: {label}"),
            Self::PostcardError(err) => write!(f, "postcard error: {err}"),
        }
    }
}

impl Error for RecvError {}

#[derive(Clone, Debug)]
pub enum TryCallError {
    SendError(postcard::Error),
    RecvError(RecvError),
}

impl fmt::Display for TryCallError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SendError(err) => write!(f, "send error: {err}"),
            Self::RecvError(err) => write!(f, "recv error: {err}"),
        }
    }
}

impl Error for TryCallError {}

// // //

fn bytes_to_words(num_bytes: usize) -> usize {
    let d = size_of::<MessageRegisterValue>();
    num_bytes.next_multiple_of(d) / d
}
