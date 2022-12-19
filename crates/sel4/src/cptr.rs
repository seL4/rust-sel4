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
    use sel4_helper_macros::CapType;

    use super::CapType;

    macro_rules! declare {
        ($t:ident) => {
            #[derive(Copy, Clone, Eq, PartialEq, CapType)]
            pub struct $t;
        };
    }

    declare!(Untyped);
    declare!(Endpoint);
    declare!(Notification);
    declare!(TCB);
    declare!(VCPU);
    declare!(CNode);
    declare!(SmallPage);
    declare!(LargePage);
    declare!(HugePage);
    declare!(PGD);
    declare!(PUD);
    declare!(PD);
    declare!(PT);
    declare!(IRQControl);
    declare!(IRQHandler);
    declare!(ASIDControl);
    declare!(ASIDPool);
    declare!(Unspecified);
    declare!(Null);
}

use local_cptr::*;

pub mod local_cptr {
    use super::{cap_type, LocalCPtr, NoExplicitInvocationContext};

    macro_rules! alias {
        ($t:ident) => {
            pub type $t<C = NoExplicitInvocationContext> = LocalCPtr<cap_type::$t, C>;
        };
    }

    alias!(Untyped);
    alias!(Endpoint);
    alias!(Notification);
    alias!(TCB);
    alias!(VCPU);
    alias!(CNode);
    alias!(SmallPage);
    alias!(LargePage);
    alias!(HugePage);
    alias!(PGD);
    alias!(PUD);
    alias!(PD);
    alias!(PT);
    alias!(IRQControl);
    alias!(IRQHandler);
    alias!(ASIDControl);
    alias!(ASIDPool);
    alias!(Unspecified);
    alias!(Null);
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

// HACK until we get negative reasoning
pub auto trait NotCNodeCapType {}

impl !NotCNodeCapType for cap_type::CNode {}
