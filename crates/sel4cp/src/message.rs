//! Utilities for handling IPC messages for protected procedure calls.

pub type MessageLabel = sel4::Word;
pub type MessageRegisterValue = sel4::Word;

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
}

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
