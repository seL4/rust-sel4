use core::cell::RefCell;

use crate::IPCBuffer;

/// A strategy for discovering the current thread's IPC buffer.
pub trait InvocationContext {
    fn invoke<T>(self, f: impl FnOnce(&mut IPCBuffer) -> T) -> T;
}

/// The absence of a strategy for discovering the current thread's IPC buffer.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct NoInvocationContext;

impl NoInvocationContext {
    pub const fn new() -> Self {
        Self
    }
}

/// The trivial of a strategy for discovering the current thread's IPC buffer.
pub type ExplicitInvocationContext<'a> = &'a mut IPCBuffer;

impl<'a> InvocationContext for ExplicitInvocationContext<'a> {
    fn invoke<T>(self, f: impl FnOnce(&mut IPCBuffer) -> T) -> T {
        f(self)
    }
}

impl InvocationContext for &RefCell<IPCBuffer> {
    fn invoke<T>(self, f: impl FnOnce(&mut IPCBuffer) -> T) -> T {
        f(&mut self.borrow_mut())
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "state")] {
        type NoExplicitInvocationContextInternal = crate::ImplicitInvocationContext;
    } else {
        type NoExplicitInvocationContextInternal = NoInvocationContext;
    }
}

/// The default strategy for discovering the current thread's IPC buffer.
///
/// When the `"state"` feature is enabled, [`NoExplicitInvocationContext`] is an alias for
/// [`ImplicitInvocationContext`](crate::ImplicitInvocationContext), which uses the [`IPCBuffer`]
/// set by [`set_ipc_buffer`](crate::set_ipc_buffer). Otherwise, it is an alias for
/// [`NoInvocationContext`](crate::NoInvocationContext), which does not implement
/// [`InvocationContext`].
pub type NoExplicitInvocationContext = NoExplicitInvocationContextInternal;
