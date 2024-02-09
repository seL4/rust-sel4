//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use core::cell::UnsafeCell;

use crate::{InvocationContext, IpcBuffer};

mod token;

#[allow(unused_imports)]
use token::{BorrowError, BorrowMutError, SyncToken, Token, UnsyncToken};

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
    fn with_context<T>(&mut self, f: impl FnOnce(&mut IpcBuffer) -> T) -> T {
        with_ipc_buffer_mut(f)
    }
}

/// Sets the IPC buffer that this crate will use for this thread.
///
/// This function does not modify kernel state. It only affects this crate's thread-local state.
///
/// Requires the `"state"` feature to be enabled.
pub fn set_ipc_buffer(ipc_buffer: &'static mut IpcBuffer) {
    try_with_ipc_buffer_slot_mut(|slot| {
        *slot.unwrap() = Some(ipc_buffer);
    })
}

/// Provides access to this thread's IPC buffer.
///
/// Requires the `"state"` feature to be enabled.
pub fn with_ipc_buffer<F, T>(f: F) -> T
where
    F: FnOnce(&IpcBuffer) -> T,
{
    try_with_ipc_buffer_slot(|buf| f(buf.unwrap().as_ref().unwrap()))
}

/// Provides mutable access to this thread's IPC buffer.
///
/// Requires the `"state"` feature to be enabled.
pub fn with_ipc_buffer_mut<F, T>(f: F) -> T
where
    F: FnOnce(&mut IpcBuffer) -> T,
{
    try_with_ipc_buffer_slot_mut(|buf| f(buf.unwrap().as_mut().unwrap()))
}

pub fn try_with_ipc_buffer_slot<F, T>(f: F) -> T
where
    F: FnOnce(Result<&Option<&'static mut IpcBuffer>, BorrowError>) -> T,
{
    let (_, r) = take_ok(TOKEN.0.try_borrow());
    f(r.map(|_| unsafe { __sel4_ipc_buffer.0.get().as_ref().unwrap() }))
}

pub fn try_with_ipc_buffer_slot_mut<F, T>(f: F) -> T
where
    F: FnOnce(Result<&mut Option<&'static mut IpcBuffer>, BorrowMutError>) -> T,
{
    let (_, r) = take_ok(TOKEN.0.try_borrow_mut());
    f(r.map(|_| unsafe { __sel4_ipc_buffer.0.get().as_mut().unwrap() }))
}

fn take_ok<T, E>(r: Result<T, E>) -> (Option<T>, Result<(), E>) {
    match r {
        Ok(ok) => (Some(ok), Ok(())),
        Err(err) => (None, Err(err)),
    }
}

pub const fn ipc_buffer_is_thread_local() -> bool {
    IPC_BUFFER_IS_THREAD_LOCAL
}

#[repr(transparent)]
struct IpcBufferSlot(UnsafeCell<Option<&'static mut IpcBuffer>>);

unsafe impl Sync for IpcBufferSlot {}

struct WrappedToken(TokenImpl);

cfg_if::cfg_if! {
    if #[cfg(all(any(target_thread_local, feature = "tls"), not(feature = "non-thread-local-state")))] {
        type TokenImpl = UnsyncToken;

        const IPC_BUFFER_IS_THREAD_LOCAL: bool = true;

        macro_rules! maybe_add_thread_local_attr {
            { $item:item } => {
                #[thread_local]
                $item
            }
        }
    } else if #[cfg(not(feature = "thread-local-state"))] {
        cfg_if::cfg_if! {
            if #[cfg(feature = "single-threaded")] {
                unsafe impl Sync for WrappedToken {}

                type TokenImpl = UnsyncToken;
            } else {
                type TokenImpl = SyncToken;
            }
        }

        const IPC_BUFFER_IS_THREAD_LOCAL: bool = false;

        macro_rules! maybe_add_thread_local_attr {
            { $item:item } => {
                $item
            }
        }
    } else {
        compile_error!(r#"invalid configuration"#);
    }
}

maybe_add_thread_local_attr! {
    static TOKEN: WrappedToken = WrappedToken(Token::INIT);
}

cfg_if::cfg_if! {
    if #[cfg(feature = "extern-state")] {
        extern "C" {
            maybe_add_thread_local_attr! {
                static __sel4_ipc_buffer: IpcBufferSlot;
            }
        }
    } else {
        maybe_add_thread_local_attr! {
            #[allow(non_upper_case_globals)]
            #[cfg_attr(feature = "exposed-state", no_mangle)]
            static __sel4_ipc_buffer: IpcBufferSlot = IpcBufferSlot(UnsafeCell::new(None));
        }
    }
}
