use core::mem;
use core::slice;

use crate::{sys, Word};

#[derive(Debug)]
pub struct IPCBuffer {
    ptr: *mut sys::seL4_IPCBuffer,
}

impl IPCBuffer {
    pub unsafe fn from_ptr(ptr: *mut sys::seL4_IPCBuffer) -> Self {
        Self { ptr }
    }

    pub fn inner(&self) -> &sys::seL4_IPCBuffer {
        unsafe { self.ptr.as_ref().unwrap() }
    }

    pub fn inner_mut(&mut self) -> &mut sys::seL4_IPCBuffer {
        unsafe { self.ptr.as_mut().unwrap() }
    }

    pub fn msg_regs(&self) -> &[Word] {
        &self.inner().msg[..]
    }

    pub fn msg_regs_mut(&mut self) -> &mut [Word] {
        &mut self.inner_mut().msg[..]
    }

    pub fn msg_bytes(&self) -> &[u8] {
        let msg = &self.inner().msg;
        let msg_ptr = msg as *const Word;
        let size = mem::size_of_val(msg);
        unsafe { slice::from_raw_parts(msg_ptr.cast(), size) }
    }

    pub fn msg_bytes_mut(&mut self) -> &mut [u8] {
        let msg = &mut self.inner_mut().msg;
        let msg_ptr = msg as *mut Word;
        let size = mem::size_of_val(msg);
        unsafe { slice::from_raw_parts_mut(msg_ptr.cast(), size) }
    }

    pub fn user_data(&self) -> Word {
        self.inner().userData
    }

    pub fn set_user_data(&mut self, data: Word) {
        self.inner_mut().userData = data;
    }
}
