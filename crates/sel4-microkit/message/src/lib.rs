//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::error::Error;
use core::fmt;

use serde::{Deserialize, Serialize};
use zerocopy::IntoBytes;

use sel4_microkit_base::{
    with_msg_regs, with_msg_regs_mut, MessageInfo, MessageLabel, MessageRegisterValue,
};

const MAX_MESSAGE_LABEL: MessageLabel =
    !0 >> (size_of::<MessageInfo>() * 8 - MessageInfo::label_width());

pub const UNSPECIFIED_ERROR_MESSAGE_LABEL: MessageLabel = MAX_MESSAGE_LABEL;

const EMPTY_MESSAGE_LABEL: MessageLabel = 0;

// // //

pub trait MessageInfoExt: Sized {
    fn send_unspecified_error() -> Self;

    fn send_empty() -> Self;

    fn recv_empty(&self) -> Result<(), RecvEmptyError>;

    fn send_using_postcard<T: Serialize>(val: T) -> Result<Self, postcard::Error>;

    fn recv_using_postcard<T: for<'a> Deserialize<'a>>(self) -> Result<T, RecvUsingPostcardError>;
}

impl MessageInfoExt for MessageInfo {
    fn send_unspecified_error() -> Self {
        Self::new(UNSPECIFIED_ERROR_MESSAGE_LABEL, 0)
    }

    fn send_empty() -> Self {
        Self::new(EMPTY_MESSAGE_LABEL, 0)
    }

    fn recv_empty(&self) -> Result<(), RecvEmptyError> {
        let label = self.label();
        if label != EMPTY_MESSAGE_LABEL {
            return Err(RecvEmptyError::UnexpectedLabel { label });
        }
        let count = self.count();
        if count != 0 {
            return Err(RecvEmptyError::UnexpectedCount { count });
        }
        Ok(())
    }

    fn send_using_postcard<T: Serialize>(val: T) -> Result<Self, postcard::Error> {
        with_msg_regs_mut(|buf| {
            let used = postcard::to_slice(&val, buf.as_mut_bytes())?;
            let count = bytes_to_words(used.len());
            Ok(Self::new(0, count))
        })
    }

    fn recv_using_postcard<T: for<'a> Deserialize<'a>>(self) -> Result<T, RecvUsingPostcardError> {
        let label = self.label();
        if label != 0 {
            return Err(RecvUsingPostcardError::UnexpectedLabel { label });
        }
        with_msg_regs(|buf| {
            postcard::from_bytes(buf.as_bytes()).map_err(RecvUsingPostcardError::PostcardError)
        })
    }
}

// // //

#[derive(Clone, Debug)]
pub enum RecvEmptyError {
    UnexpectedLabel { label: MessageLabel },
    UnexpectedCount { count: usize },
}

impl fmt::Display for RecvEmptyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedLabel { label } => write!(f, "unexpected label: {label}"),
            Self::UnexpectedCount { count } => write!(f, "unexpected count: {count}"),
        }
    }
}

impl Error for RecvEmptyError {}

#[derive(Clone, Debug)]
pub enum RecvUsingPostcardError {
    UnexpectedLabel { label: MessageLabel },
    PostcardError(postcard::Error),
}

impl fmt::Display for RecvUsingPostcardError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedLabel { label } => write!(f, "unexpected label: {label}"),
            Self::PostcardError(err) => write!(f, "postcard error: {err}"),
        }
    }
}

impl Error for RecvUsingPostcardError {}

// // //

fn bytes_to_words(num_bytes: usize) -> usize {
    let d = size_of::<MessageRegisterValue>();
    num_bytes.next_multiple_of(d) / d
}
