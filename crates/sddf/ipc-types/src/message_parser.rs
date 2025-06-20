//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::error::Error;
use core::fmt;

use crate::{MessageLabel, MessageRegisterValue};

pub struct MessageParser<'a> {
    label: MessageLabel,
    buf: &'a [MessageRegisterValue],
}

impl<'a> MessageParser<'a> {
    pub fn new(label: MessageLabel, buf: &'a [MessageRegisterValue]) -> Self {
        Self { label, buf }
    }

    pub fn label(&self) -> MessageLabel {
        self.label
    }

    pub fn label_try_into<T: TryFrom<MessageLabel>>(&self) -> Result<T, MessagParseError> {
        let label = self.label();
        label
            .try_into()
            .map_err(|_| MessagParseError::UnexpectedLabel { label })
    }

    pub fn ensure_label_eq(&self, expected: MessageLabel) -> Result<(), MessagParseError> {
        let label = self.label();
        if label == expected {
            Ok(())
        } else {
            Err(MessagParseError::UnexpectedLabel { label })
        }
    }

    pub fn get_mr<T: TryFromMessageRegisterValue>(&self, i: usize) -> Result<T, MessagParseError> {
        let val = self.buf.get(i).ok_or(MessagParseError::MessageTooShort {
            index: i,
            length: self.buf.len(),
        })?;
        T::try_from_message_register_value(*val).ok_or(
            MessagParseError::InvalidMessageRegisterValue {
                index: i,
                value: *val,
            },
        )
    }
}

#[derive(Clone, Debug)]
pub enum MessagParseError {
    UnexpectedLabel {
        label: MessageLabel,
    },
    MessageTooShort {
        index: usize,
        length: usize,
    },
    InvalidMessageRegisterValue {
        index: usize,
        value: MessageRegisterValue,
    },
}

impl fmt::Display for MessagParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedLabel { label } => write!(f, "unexpected label: {label}"),
            Self::MessageTooShort { index, length } => {
                write!(f, "message of length {length} too short for index {index}")
            }
            Self::InvalidMessageRegisterValue { index, value } => write!(
                f,
                "invalid message register value at index {index}: {value:#x}"
            ),
        }
    }
}

impl Error for MessagParseError {}

pub trait TryFromMessageRegisterValue: Sized {
    fn try_from_message_register_value(val: MessageRegisterValue) -> Option<Self>;
}

impl TryFromMessageRegisterValue for MessageRegisterValue {
    fn try_from_message_register_value(val: MessageRegisterValue) -> Option<Self> {
        Some(val)
    }
}
