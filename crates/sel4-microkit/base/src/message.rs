//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

pub type MessageLabel = sel4::Word;
pub type MessageRegisterValue = sel4::Word;

#[derive(Debug, Clone)]
pub struct MessageInfo {
    inner: sel4::MessageInfo,
}

impl MessageInfo {
    #[doc(hidden)]
    pub fn from_inner(inner: sel4::MessageInfo) -> Self {
        Self { inner }
    }

    #[doc(hidden)]
    pub fn into_inner(self) -> sel4::MessageInfo {
        self.inner
    }

    pub fn new(label: MessageLabel, count: usize) -> Self {
        Self::from_inner(sel4::MessageInfo::new(label, 0, 0, count))
    }

    pub fn label(&self) -> MessageLabel {
        self.inner.label()
    }

    pub const fn label_width() -> usize {
        sel4::MessageInfo::label_width()
    }

    pub fn count(&self) -> usize {
        self.inner.length()
    }
}

impl Default for MessageInfo {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

pub fn with_msg_regs<T>(f: impl FnOnce(&[MessageRegisterValue]) -> T) -> T {
    sel4::with_ipc_buffer(|ipc_buffer| f(ipc_buffer.msg_regs()))
}

pub fn with_msg_regs_mut<T>(f: impl FnOnce(&mut [MessageRegisterValue]) -> T) -> T {
    sel4::with_ipc_buffer_mut(|ipc_buffer| f(ipc_buffer.msg_regs_mut()))
}

pub fn with_msg_bytes<T>(f: impl FnOnce(&[u8]) -> T) -> T {
    sel4::with_ipc_buffer(|ipc_buffer| f(ipc_buffer.msg_bytes()))
}

pub fn with_msg_bytes_mut<T>(f: impl FnOnce(&mut [u8]) -> T) -> T {
    sel4::with_ipc_buffer_mut(|ipc_buffer| f(ipc_buffer.msg_bytes_mut()))
}

pub fn set_mr(i: usize, value: MessageRegisterValue) {
    with_msg_regs_mut(|regs| regs[i] = value)
}

pub fn get_mr(i: usize) -> MessageRegisterValue {
    with_msg_regs(|regs| regs[i])
}
