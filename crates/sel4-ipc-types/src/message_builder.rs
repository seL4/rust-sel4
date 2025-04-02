//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use crate::{MessageLabel, MessageRegisterValue};

pub struct MessageBuilder<'a> {
    label: MessageLabel,
    buf: &'a mut [MessageRegisterValue],
    length: usize,
}

impl<'a> MessageBuilder<'a> {
    pub fn new(buf: &'a mut [MessageRegisterValue]) -> Self {
        Self {
            label: 0,
            buf,
            length: 0,
        }
    }

    pub fn set_label(&mut self, label: MessageLabel) {
        self.label = label;
    }

    pub fn set_mr(&mut self, i: usize, val: impl IntoMessageRegisterValue) {
        self.buf[self.length..i].fill(0);
        self.buf[i] = val.into_message_register_value();
        self.length = i + 1;
    }

    pub fn set_next_mr(&mut self, val: impl IntoMessageRegisterValue) {
        let i = self.length;
        self.set_mr(i, val);
    }

    pub fn build(self) -> (MessageLabel, usize) {
        (self.label, self.length)
    }
}

pub trait IntoMessageRegisterValue {
    fn into_message_register_value(self) -> MessageRegisterValue;
}

impl IntoMessageRegisterValue for MessageRegisterValue {
    fn into_message_register_value(self) -> MessageRegisterValue {
        self
    }
}
