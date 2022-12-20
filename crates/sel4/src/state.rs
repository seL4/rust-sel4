use core::cell::RefCell;

use crate::{IPCBuffer, InvocationContext};

const IPC_BUFFER_INIT: RefCell<Option<IPCBuffer>> = RefCell::new(None);

cfg_if::cfg_if! {
    if #[cfg(feature = "tls")] {
        #[thread_local]
        pub static IPC_BUFFER: RefCell<Option<IPCBuffer>> = IPC_BUFFER_INIT;
    } else if #[cfg(feature = "single-threaded")] {
        use core::ops::{Deref, DerefMut};

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

pub unsafe fn set_ipc_buffer(ipc_buffer: IPCBuffer) {
    let _ = IPC_BUFFER.replace(Some(ipc_buffer));
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

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct ImplicitInvocationContext;

impl ImplicitInvocationContext {
    pub const fn new() -> Self {
        Self
    }
}

impl InvocationContext for ImplicitInvocationContext {
    fn invoke<T>(self, f: impl FnOnce(&mut IPCBuffer) -> T) -> T {
        with_ipc_buffer_mut(f)
    }
}
