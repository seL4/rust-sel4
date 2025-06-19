//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::convert::Infallible;

use crate::{
    MessagParseError, MessageBuilder, MessageLabel, MessageParser, MessageRegisterValue,
    MessageWriter, ReadFromMessage,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct EmptyMessage(());

impl EmptyMessage {
    pub const fn new() -> Self {
        Self(())
    }
}

impl MessageWriter for EmptyMessage {
    type Error = Infallible;

    fn write_message(
        &self,
        buf: &mut [MessageRegisterValue],
    ) -> Result<(MessageLabel, usize), Self::Error> {
        Ok(MessageBuilder::new(buf).build())
    }
}

impl ReadFromMessage for EmptyMessage {
    type Error = MessagParseError;

    fn read_from_message(
        label: MessageLabel,
        buf: &[MessageRegisterValue],
    ) -> Result<Self, Self::Error> {
        let msg = MessageParser::new(label, buf);
        msg.ensure_label_eq(0)?;
        Ok(Self::new())
    }
}
