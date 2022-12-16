use core::cell::RefCell;
use core::mem;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::slice;

use crate::{sys, Word};

#[thread_local]
pub static IPC_BUFFER: RefCell<IPCBuffer> = RefCell::new(IPCBuffer { ptr: None });

pub unsafe fn set_ipc_buffer_ptr(ptr: NonNull<sys::seL4_IPCBuffer>) {
    IPC_BUFFER.borrow_mut().ptr = Some(ptr);
}

pub struct IPCBuffer {
    ptr: Option<NonNull<sys::seL4_IPCBuffer>>,
}

impl IPCBuffer {
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

pub fn with_ipc_buffer<F, T>(f: F) -> T
where
    F: FnOnce(&IPCBuffer) -> T,
{
    f(&IPC_BUFFER.borrow())
}

pub fn with_ipc_buffer_mut<F, T>(f: F) -> T
where
    F: FnOnce(&mut IPCBuffer) -> T,
{
    f(&mut IPC_BUFFER.borrow_mut())
}
