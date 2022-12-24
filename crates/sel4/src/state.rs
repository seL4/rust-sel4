use core::cell::RefCell;

use crate::{IPCBuffer, InvocationContext};

const IPC_BUFFER_INIT: RefCell<Option<IPCBuffer>> = RefCell::new(None);

cfg_if::cfg_if! {
    if #[cfg(not(feature = "single-threaded"))] {
        #[thread_local]
        static IPC_BUFFER: RefCell<Option<IPCBuffer>> = IPC_BUFFER_INIT;

        fn with_ipc_buffer_internal<F, T>(f: F) -> T
        where
            F: FnOnce(&RefCell<Option<IPCBuffer>>) -> T,
        {
            f(&IPC_BUFFER)
        }
    } else {
        static IPC_BUFFER: SingleThreaded<RefCell<Option<IPCBuffer>>> = SingleThreaded(IPC_BUFFER_INIT);

        struct SingleThreaded<T>(T);

        unsafe impl<T> Sync for SingleThreaded<T> {}

        fn with_ipc_buffer_internal<F, T>(f: F) -> T
        where
            F: FnOnce(&RefCell<Option<IPCBuffer>>) -> T,
        {
            f(&IPC_BUFFER.0)
        }
    }
}

/// Provides access to this thread's IPC buffer.
///
/// This function is a convenience wrapper around [`with_ipc_buffer`].
///
/// Requires the `"state"` feature to be enabled.
pub fn with_ipc_buffer<F, T>(f: F) -> T
where
    F: FnOnce(&RefCell<Option<IPCBuffer>>) -> T,
{
    with_ipc_buffer_internal(f)
}

/// Sets the IPC buffer that this crate will use for this thread.
///
/// This function does not modify kernel state. It only this crate's thread-local state.
///
/// This function is a convenience wrapper around [`with_ipc_buffer`].
///
/// Requires the `"state"` feature to be enabled.
pub unsafe fn set_ipc_buffer(ipc_buffer: IPCBuffer) {
    with_ipc_buffer(|buf| {
        let _ = buf.replace(Some(ipc_buffer));
    })
}

/// Provides access to a borrowed reference to this thread's IPC buffer.
///
/// This function is a convenience wrapper around [`with_ipc_buffer`].
///
/// Requires the `"state"` feature to be enabled.
pub fn with_borrow_ipc_buffer<F, T>(f: F) -> T
where
    F: FnOnce(&IPCBuffer) -> T,
{
    with_ipc_buffer(|buf| f(buf.borrow().as_ref().unwrap()))
}

/// Provides access to a mutably borrowed reference to this thread's IPC buffer.
///
/// Requires the `"state"` feature to be enabled.
pub fn with_borrow_ipc_buffer_mut<F, T>(f: F) -> T
where
    F: FnOnce(&mut IPCBuffer) -> T,
{
    with_ipc_buffer(|buf| f(buf.borrow_mut().as_mut().unwrap()))
}

/// The strategy for discovering the current thread's IPC buffer which uses thread-local state.
///
/// This thread-local state can be modified using [`with_ipc_buffer`] and [`set_ipc_buffer`].
///
/// Requires the `"state"` feature to be enabled.
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
