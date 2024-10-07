//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

/// Type alias for [`MessageInfo`] labels.
pub type MessageLabel = sel4::Word;

/// Type alias for message register values.
pub type MessageRegisterValue = sel4::Word;

/// Corresponds to `microkit_msginfo`.
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
    pub fn inner(&self) -> &sel4::MessageInfo {
        &self.inner
    }

    #[doc(hidden)]
    pub fn into_inner(self) -> sel4::MessageInfo {
        self.inner
    }

    pub fn new(label: MessageLabel, count: usize) -> Self {
        Self::from_inner(sel4::MessageInfo::new(label, 0, 0, count))
    }

    /// The label associated with this message.
    pub fn label(&self) -> MessageLabel {
        self.inner.label()
    }

    /// The number of meaningful bits in `MessageLabel`.
    pub const fn label_width() -> usize {
        sel4::MessageInfo::label_width()
    }

    /// The number of filled message registers associated with this message.
    pub fn count(&self) -> usize {
        self.inner.length()
    }

    /// Interpret this message as a [`sel4::Fault`].
    pub fn fault(&self) -> sel4::Fault {
        sel4::with_ipc_buffer(|ipc_buffer| sel4::Fault::new(ipc_buffer, self.inner()))
    }
}

impl Default for MessageInfo {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

/// Provides access to the protection domain's message registers.
pub fn with_msg_regs<T>(f: impl FnOnce(&[MessageRegisterValue]) -> T) -> T {
    sel4::with_ipc_buffer(|ipc_buffer| f(ipc_buffer.msg_regs()))
}

/// Provides mutable access to the protection domain's message registers.
pub fn with_msg_regs_mut<T>(f: impl FnOnce(&mut [MessageRegisterValue]) -> T) -> T {
    sel4::with_ipc_buffer_mut(|ipc_buffer| f(ipc_buffer.msg_regs_mut()))
}

/// Provides access to the protection domain's message registers, viewed as an array of bytes.
pub fn with_msg_bytes<T>(f: impl FnOnce(&[u8]) -> T) -> T {
    sel4::with_ipc_buffer(|ipc_buffer| f(ipc_buffer.msg_bytes()))
}

/// Provides mutable access to the protection domain's message registers, viewed as an array of
/// bytes.
pub fn with_msg_bytes_mut<T>(f: impl FnOnce(&mut [u8]) -> T) -> T {
    sel4::with_ipc_buffer_mut(|ipc_buffer| f(ipc_buffer.msg_bytes_mut()))
}

/// Corresponds to `microkit_mr_set`.
pub fn set_mr(i: usize, value: MessageRegisterValue) {
    with_msg_regs_mut(|regs| regs[i] = value)
}

/// Corresponds to `microkit_mr_get`.
pub fn get_mr(i: usize) -> MessageRegisterValue {
    with_msg_regs(|regs| regs[i])
}
