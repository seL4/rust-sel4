use core::mem;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::slice;

use crate::{sys, Word};

pub struct IPCBuffer {
    ptr: Option<NonNull<sys::seL4_IPCBuffer>>,
}

impl IPCBuffer {
    pub const fn unset() -> Self {
        Self {
            ptr: None,
        }
    }

    pub unsafe fn set_ptr(&mut self, ptr: NonNull<sys::seL4_IPCBuffer>) {
        self.ptr = Some(ptr)
    }

    fn inner(&self) -> &sys::seL4_IPCBuffer {
        unsafe { self.ptr.unwrap().as_ref() }
    }

    fn inner_mut(&mut self) -> &mut sys::seL4_IPCBuffer {
        unsafe { self.ptr.unwrap().as_mut() }
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

impl Deref for IPCBuffer {
    type Target = sys::seL4_IPCBuffer_;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl DerefMut for IPCBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner_mut()
    }
}
