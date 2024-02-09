//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use core::fmt;

use crate::{IPCBuffer, InvocationContext};

// For the sake of consistent behavior between configurations, re-entrancy is not supported even in
// the immutable case

cfg_if::cfg_if! {
    if #[cfg(target_thread_local)] {
        use core::cell::RefCell;

        #[thread_local]
        static IPC_BUFFER: RefCell<Option<&'static mut IPCBuffer>> = RefCell::new(None);

        fn try_with_ipc_buffer_internal<F, T>(f: F) -> T
        where
            F: FnOnce(Result<&mut Option<&'static mut IPCBuffer>, BorrowError>) -> T,
        {
            match IPC_BUFFER.try_borrow_mut() {
                Ok(mut buf) => f(Ok(&mut *buf)),
                Err(_) => f(Err(BorrowError::new())),
            }
        }
    } else if #[cfg(feature = "single-threaded")] {
        use core::sync::atomic::{AtomicBool, Ordering};

        static mut IPC_BUFFER: Option<&'static mut IPCBuffer> = None;

        static IPC_BUFFER_BORROWED: AtomicBool = AtomicBool::new(false);

        fn try_with_ipc_buffer_internal<F, T>(f: F) -> T
        where
            F: FnOnce(Result<&mut Option<&'static mut IPCBuffer>, BorrowError>) -> T,
        {
            // release on panic
            struct Guard;

            impl Drop for Guard {
                fn drop(&mut self) {
                    IPC_BUFFER_BORROWED.store(false, Ordering::Release);
                }
            }

            if IPC_BUFFER_BORROWED.swap(true, Ordering::Acquire) {
                f(Err(BorrowError::new()))
            } else {
                let _ = Guard;
                unsafe {
                    f(Ok(&mut IPC_BUFFER))
                }
            }
        }
    } else {
        compile_error!(r#"when #[cfg(feature = "state")], at least one of #[cfg(target_thread_local)] or #[cfg(feature = "single-threaded")] is required"#);
    }
}

fn try_with_ipc_buffer_raw<F, T>(f: F) -> T
where
    F: FnOnce(Result<&mut Option<&'static mut IPCBuffer>, BorrowError>) -> T,
{
    try_with_ipc_buffer_internal(f)
}

#[derive(Debug, Clone)]
pub struct BorrowError(());

impl BorrowError {
    fn new() -> Self {
        Self(())
    }
}

impl fmt::Display for BorrowError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IPC buffer already borrowed")
    }
}

fn with_ipc_buffer_raw<F, T>(f: F) -> T
where
    F: FnOnce(&mut Option<&'static mut IPCBuffer>) -> T,
{
    try_with_ipc_buffer_raw(|r| f(r.unwrap()))
}

/// Sets the IPC buffer that this crate will use for this thread.
///
/// This function does not modify kernel state. It only affects this crate's thread-local state.
///
/// Requires the `"state"` feature to be enabled.
pub fn set_ipc_buffer(ipc_buffer: &'static mut IPCBuffer) {
    with_ipc_buffer_raw(|buf| {
        let _ = buf.replace(ipc_buffer);
    })
}

/// Provides access to this thread's IPC buffer.
///
/// Requires the `"state"` feature to be enabled.
pub fn with_ipc_buffer<F, T>(f: F) -> T
where
    F: FnOnce(&IPCBuffer) -> T,
{
    with_ipc_buffer_raw(|buf| f(buf.as_ref().unwrap()))
}

/// Provides mutable access to this thread's IPC buffer.
///
/// Requires the `"state"` feature to be enabled.
pub fn with_ipc_buffer_mut<F, T>(f: F) -> T
where
    F: FnOnce(&mut IPCBuffer) -> T,
{
    with_ipc_buffer_raw(|buf| f(buf.as_mut().unwrap()))
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
    fn invoke<T>(&mut self, f: impl FnOnce(&mut IPCBuffer) -> T) -> T {
        with_ipc_buffer_mut(f)
    }
}
