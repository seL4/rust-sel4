//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use serde::{Deserialize, Serialize};

use crate::{
    MessageRegisters, MessageRegistersMut, MessageRegistersPrefixLength, MessageValueRecv,
    MessageValueSend,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MessageValueUsingPostcard<T>(pub T);

impl<T: Serialize> MessageValueSend for MessageValueUsingPostcard<T> {
    type Error = postcard::Error;

    fn write_message_value(
        &self,
        regs: MessageRegistersMut,
    ) -> Result<MessageRegistersPrefixLength, Self::Error> {
        regs.with_bytes(|buf| postcard::to_slice(&self.0, buf).map(|used| used.len()))
    }
}

impl<T: for<'a> Deserialize<'a>> MessageValueRecv for MessageValueUsingPostcard<T> {
    type Error = postcard::Error;

    fn read_message_value(regs: &MessageRegisters) -> Result<Self, Self::Error> {
        postcard::from_bytes(regs.as_bytes()).map(MessageValueUsingPostcard)
    }
}
