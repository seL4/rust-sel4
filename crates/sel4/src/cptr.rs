use core::fmt;
use core::marker::PhantomData;

use crate::{sys, IPCBuffer, InvocationContext, NoExplicitInvocationContext, Result, WORD_SIZE};

pub type CPtrBits = sys::seL4_CPtr;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CPtr {
    bits: CPtrBits,
}

impl CPtr {
    pub const fn bits(self) -> CPtrBits {
        self.bits
    }

    pub const fn from_bits(bits: CPtrBits) -> Self {
        Self { bits }
    }

    pub const fn cast<T: CapType>(self) -> LocalCPtr<T> {
        LocalCPtr::from_cptr(self)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LocalCPtr<T: CapType, C = NoExplicitInvocationContext> {
    phantom: PhantomData<T>,
    cptr: CPtr,
    invocation_context: C,
}

impl<T: CapType, C> LocalCPtr<T, C> {
    pub const fn cptr(&self) -> CPtr {
        self.cptr
    }

    pub const fn bits(&self) -> CPtrBits {
        self.cptr().bits()
    }

    pub fn cast<T1: CapType>(self) -> LocalCPtr<T1, C> {
        LocalCPtr {
            phantom: PhantomData,
            cptr: self.cptr,
            invocation_context: self.invocation_context,
        }
    }

    pub fn with<C1>(self, context: C1) -> LocalCPtr<T, C1> {
        LocalCPtr {
            phantom: self.phantom,
            cptr: self.cptr,
            invocation_context: context,
        }
    }

    pub fn without_context(self) -> LocalCPtr<T> {
        self.with(NoExplicitInvocationContext::new())
    }
}

impl<T: CapType> LocalCPtr<T> {
    pub const fn from_cptr(cptr: CPtr) -> Self {
        Self {
            phantom: PhantomData,
            cptr,
            invocation_context: NoExplicitInvocationContext::new(),
        }
    }

    pub const fn from_bits(bits: CPtrBits) -> Self {
        CPtr::from_bits(bits).cast()
    }
}

impl<T: CapType, C: InvocationContext> LocalCPtr<T, C> {
    pub fn invoke<R>(self, f: impl FnOnce(CPtr, &mut IPCBuffer) -> R) -> R {
        let cptr = self.cptr();
        self.invocation_context
            .invoke(|ipc_buffer| f(cptr, ipc_buffer))
    }
}

impl<T: CapType, C> fmt::Debug for LocalCPtr<T, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple(T::NAME).field(&self.cptr().bits()).finish()
    }
}

// NOTE require 'Copy' for convenience to make up for limitations of automatic trait derivation
pub trait CapType: Copy {
    const NAME: &'static str;
}

pub mod cap_type {
    use crate::declare_cap_type;

    pub use crate::arch::cap_type_arch::*;

    declare_cap_type!(Untyped);
    declare_cap_type!(Endpoint);
    declare_cap_type!(Notification);
    declare_cap_type!(TCB);
    declare_cap_type!(CNode);
    declare_cap_type!(IRQControl);
    declare_cap_type!(IRQHandler);
    declare_cap_type!(ASIDControl);
    declare_cap_type!(ASIDPool);

    declare_cap_type!(Null);
    declare_cap_type!(Unspecified);
}

use local_cptr::*;

pub mod local_cptr {
    use crate::declare_local_cptr_alias;

    pub use crate::arch::local_cptr_arch::*;

    declare_local_cptr_alias!(Untyped);
    declare_local_cptr_alias!(Endpoint);
    declare_local_cptr_alias!(Notification);
    declare_local_cptr_alias!(TCB);
    declare_local_cptr_alias!(CNode);
    declare_local_cptr_alias!(IRQControl);
    declare_local_cptr_alias!(IRQHandler);
    declare_local_cptr_alias!(ASIDControl);
    declare_local_cptr_alias!(ASIDPool);

    declare_local_cptr_alias!(Null);
    declare_local_cptr_alias!(Unspecified);

    declare_local_cptr_alias!(VSpace);
    declare_local_cptr_alias!(Granule);
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CPtrWithDepth {
    bits: CPtrBits,
    depth: usize,
}

impl CPtrWithDepth {
    pub const fn from_bits_with_depth(bits: CPtrBits, depth: usize) -> Self {
        Self { bits, depth }
    }

    pub const fn bits(&self) -> CPtrBits {
        self.bits
    }

    pub const fn depth(&self) -> usize {
        self.depth
    }

    pub const fn empty() -> Self {
        Self::from_bits_with_depth(0, 0)
    }

    // convenience
    pub(crate) fn depth_for_kernel(&self) -> u8 {
        self.depth().try_into().unwrap()
    }
}

impl From<CPtr> for CPtrWithDepth {
    fn from(cptr: CPtr) -> Self {
        Self::from_bits_with_depth(cptr.bits(), WORD_SIZE)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RelativeCPtr<C = NoExplicitInvocationContext> {
    root: CNode<C>,
    path: CPtrWithDepth,
}

impl<C> RelativeCPtr<C> {
    pub const fn root(&self) -> &CNode<C> {
        &self.root
    }

    pub const fn path(&self) -> &CPtrWithDepth {
        &self.path
    }

    pub fn with<C1>(self, context: C1) -> RelativeCPtr<C1> {
        RelativeCPtr {
            root: self.root.with(context),
            path: self.path,
        }
    }

    pub fn without_context(self) -> RelativeCPtr {
        self.with(NoExplicitInvocationContext::new())
    }
}

impl<C: InvocationContext> RelativeCPtr<C> {
    pub fn invoke<R>(self, f: impl FnOnce(CPtr, CPtrWithDepth, &mut IPCBuffer) -> R) -> R {
        let path = *self.path();
        self.root
            .invoke(|cptr, ipc_buffer| f(cptr, path, ipc_buffer))
    }
}

pub trait HasCPtrWithDepth {
    fn cptr_with_depth(self) -> CPtrWithDepth;
}

impl HasCPtrWithDepth for CPtr {
    fn cptr_with_depth(self) -> CPtrWithDepth {
        self.into()
    }
}

impl<T: CapType, C> HasCPtrWithDepth for LocalCPtr<T, C> {
    fn cptr_with_depth(self) -> CPtrWithDepth {
        self.cptr().into()
    }
}

impl HasCPtrWithDepth for CPtrWithDepth {
    fn cptr_with_depth(self) -> CPtrWithDepth {
        self
    }
}

impl<C> CNode<C> {
    pub fn relative<T: HasCPtrWithDepth>(self, path: T) -> RelativeCPtr<C> {
        RelativeCPtr {
            root: self,
            path: path.cptr_with_depth(),
        }
    }

    pub fn relative_bits_with_depth(self, bits: CPtrBits, depth: usize) -> RelativeCPtr<C> {
        self.relative(CPtrWithDepth::from_bits_with_depth(bits, depth))
    }

    pub fn relative_self(self) -> RelativeCPtr<C> {
        self.relative(CPtrWithDepth::empty())
    }
}

impl<C: InvocationContext> CNode<C> {
    pub fn save_caller(self, ep: Endpoint) -> Result<()> {
        self.relative(ep).save_caller()
    }
}

impl<C> Unspecified<C> {
    pub fn downcast<T: CapType>(self) -> LocalCPtr<T, C> {
        self.cast()
    }
}
