//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use core::cell::UnsafeCell;

use crate::{InvocationContext, IpcBuffer};

mod token;

#[allow(unused_imports)]
use token::{Accessor, BorrowError, BorrowMutError, SyncToken, TokenCell, UnsyncToken};

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

/// Provides low-level access to this thread's IPC buffer.
///
/// This function does not modify kernel state. It only affects this crate's thread-local state.
///
/// Requires the `"state"` feature to be enabled.
pub fn try_with_ipc_buffer_slot<F, T>(f: F) -> T
where
    F: FnOnce(Result<&Option<&'static mut IpcBuffer>, BorrowError>) -> T,
{
    IPC_BUFFER.0.try_with(f)
}

/// Provides low-level mutable access to this thread's IPC buffer.
///
/// This function does not modify kernel state. It only affects this crate's thread-local state.
///
/// Requires the `"state"` feature to be enabled.
pub fn try_with_ipc_buffer_slot_mut<F, T>(f: F) -> T
where
    F: FnOnce(Result<&mut Option<&'static mut IpcBuffer>, BorrowMutError>) -> T,
{
    IPC_BUFFER.0.try_with_mut(f)
}

/// Returns whether this crate's IPC buffer slot is thread-local.
///
/// Requires the `"state"` feature to be enabled.
pub const fn ipc_buffer_is_thread_local() -> bool {
    STATE_IS_THREAD_LOCAL
}

#[repr(transparent)]
struct SyncUnsafeCell<T>(UnsafeCell<T>);

unsafe impl<T: Sync> Sync for SyncUnsafeCell<T> {}

#[repr(transparent)]
struct TokenCellWrapper<A>(TokenCell<TokenImpl, A>);

cfg_if::cfg_if! {
    if #[cfg(all(any(target_thread_local, feature = "tls"), not(feature = "non-thread-local-state")))] {
        type TokenImpl = UnsyncToken;

        const STATE_IS_THREAD_LOCAL: bool = true;

        macro_rules! maybe_add_thread_local_attr {
            { $item:item } => {
                #[thread_local]
                $item
            }
        }
    } else if #[cfg(not(feature = "thread-local-state"))] {
        cfg_if::cfg_if! {
            if #[cfg(feature = "single-threaded")] {
                unsafe impl<A> Sync for TokenCellWrapper<A> {}

                type TokenImpl = UnsyncToken;
            } else {
                type TokenImpl = SyncToken;
            }
        }

        const STATE_IS_THREAD_LOCAL: bool = false;

        macro_rules! maybe_add_thread_local_attr {
            { $item:item } => {
                $item
            }
        }
    } else {
        compile_error!(r#"invalid configuration"#);
    }
}

macro_rules! maybe_extern {
    { $ident:ident: $ty:ty = $init:expr; } => {
        cfg_if::cfg_if! {
            if #[cfg(feature = "extern-state")] {
                extern "C" {
                    maybe_add_thread_local_attr! {
                        static $ident: $ty;
                    }
                }
            } else {
                maybe_add_thread_local_attr! {
                    #[allow(non_upper_case_globals)]
                    #[cfg_attr(feature = "exposed-state", no_mangle)]
                    static $ident: $ty = $init;
                }
            }
        }
    }
}

struct IpcBufferAccessor;

impl Accessor<Option<&'static mut IpcBuffer>> for IpcBufferAccessor {
    #[allow(unused_unsafe)]
    fn with<F, U>(&self, f: F) -> U
    where
        F: FnOnce(&UnsafeCell<Option<&'static mut IpcBuffer>>) -> U,
    {
        f(unsafe { &__sel4_ipc_buffer.0 })
    }
}

maybe_add_thread_local_attr! {
    static IPC_BUFFER: TokenCellWrapper<IpcBufferAccessor> = unsafe {
        TokenCellWrapper(TokenCell::new(IpcBufferAccessor))
    };
}

maybe_extern! {
    __sel4_ipc_buffer: SyncUnsafeCell<Option<&'static mut IpcBuffer>> =
        SyncUnsafeCell(UnsafeCell::new(None));
}
