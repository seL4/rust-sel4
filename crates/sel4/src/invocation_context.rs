use core::ptr::NonNull;
use core::cell::RefCell;

#[allow(unused_imports)]
use core::ops::{Deref, DerefMut};

use crate::{sys, IPCBuffer};

const IPC_BUFFER_INIT: RefCell<IPCBuffer> = RefCell::new(IPCBuffer::unset());

cfg_if::cfg_if! {
    if #[cfg(not(feature = "single-threaded"))] {
        #[thread_local]
        pub static IPC_BUFFER: RefCell<IPCBuffer> = IPC_BUFFER_INIT;
    } else {
        pub static IPC_BUFFER: SingleThreaded<RefCell<IPCBuffer>> = SingleThreaded(IPC_BUFFER_INIT);

        pub struct SingleThreaded<T>(pub T);

        unsafe impl<T> Sync for SingleThreaded<T> {}

        impl<T> Deref for SingleThreaded<T> {
            type Target = T;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<T> DerefMut for SingleThreaded<T> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    }
}

pub unsafe fn set_ipc_buffer_ptr(ptr: NonNull<sys::seL4_IPCBuffer>) {
    IPC_BUFFER.borrow_mut().set_ptr(ptr);
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
