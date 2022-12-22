use core::cell::RefCell;

use crate::IPCBuffer;

pub trait InvocationContext {
    fn invoke<T>(self, f: impl FnOnce(&mut IPCBuffer) -> T) -> T;
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct NoInvocationContext;

impl NoInvocationContext {
    pub const fn new() -> Self {
        Self
    }
}

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
        use crate::ImplicitInvocationContext;

        pub type NoExplicitInvocationContext = ImplicitInvocationContext;
    } else {
        pub type NoExplicitInvocationContext = NoInvocationContext;
    }
}
