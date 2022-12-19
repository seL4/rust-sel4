use core::cell::RefCell;

#[allow(unused_imports)]
use core::ops::{Deref, DerefMut};

use crate::{sys, IPCBuffer};

const IPC_BUFFER_INIT: RefCell<Option<IPCBuffer>> = RefCell::new(None);

cfg_if::cfg_if! {
    if #[cfg(feature = "tls")] {
        #[thread_local]
        pub static IPC_BUFFER: RefCell<Option<IPCBuffer>> = IPC_BUFFER_INIT;
    } else if #[cfg(feature = "single-threaded")] {
        pub static IPC_BUFFER: SingleThreaded<RefCell<Option<IPCBuffer>>> = SingleThreaded(IPC_BUFFER_INIT);

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
    } else {
        compile_error!("The feature \"state\" requires at least one of \"tls\" or \"single-threaded\"");
    }
}

pub unsafe fn set_ipc_buffer_ptr(ptr: *mut sys::seL4_IPCBuffer) {
    let _ = IPC_BUFFER.replace(Some(IPCBuffer::from_ptr(ptr)));
}

pub fn with_ipc_buffer<F, T>(f: F) -> T
where
    F: FnOnce(&IPCBuffer) -> T,
{
    f(IPC_BUFFER.borrow().as_ref().unwrap())
}

pub fn with_ipc_buffer_mut<F, T>(f: F) -> T
where
    F: FnOnce(&mut IPCBuffer) -> T,
{
    f(IPC_BUFFER.borrow_mut().as_mut().unwrap())
}
