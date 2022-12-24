use core::cell::RefCell;

use crate::{IPCBuffer, InvocationContext};

const IPC_BUFFER_INIT: RefCell<Option<IPCBuffer>> = RefCell::new(None);

cfg_if::cfg_if! {
    if #[cfg(not(feature = "single-threaded"))] {
        #[thread_local]
        static IPC_BUFFER: RefCell<Option<IPCBuffer>> = IPC_BUFFER_INIT;

        pub fn with_ipc_buffer<F, T>(f: F) -> T
        where
            F: FnOnce(&RefCell<Option<IPCBuffer>>) -> T,
        {
            f(&IPC_BUFFER)
        }
    } else {
        static IPC_BUFFER: SingleThreaded<RefCell<Option<IPCBuffer>>> = SingleThreaded(IPC_BUFFER_INIT);

        struct SingleThreaded<T>(T);

        unsafe impl<T> Sync for SingleThreaded<T> {}

        pub fn with_ipc_buffer<F, T>(f: F) -> T
        where
            F: FnOnce(&RefCell<Option<IPCBuffer>>) -> T,
        {
            f(&IPC_BUFFER.0)
        }
    }
}

pub unsafe fn set_ipc_buffer(ipc_buffer: IPCBuffer) {
    with_ipc_buffer(|buf| {
        let _ = buf.replace(Some(ipc_buffer));
    })
}

pub fn with_borrow_ipc_buffer<F, T>(f: F) -> T
where
    F: FnOnce(&IPCBuffer) -> T,
{
    with_ipc_buffer(|buf| f(buf.borrow().as_ref().unwrap()))
}

pub fn with_borrow_ipc_buffer_mut<F, T>(f: F) -> T
where
    F: FnOnce(&mut IPCBuffer) -> T,
{
    with_ipc_buffer(|buf| f(buf.borrow_mut().as_mut().unwrap()))
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct ImplicitInvocationContext;

impl ImplicitInvocationContext {
    pub const fn new() -> Self {
        Self
    }
}

impl InvocationContext for ImplicitInvocationContext {
    fn invoke<T>(self, f: impl FnOnce(&mut IPCBuffer) -> T) -> T {
        with_borrow_ipc_buffer_mut(f)
    }
}
