use core::fmt;
use core::marker::PhantomData;

use sel4_config::sel4_cfg;

use crate::{sys, IPCBuffer, InvocationContext, NoExplicitInvocationContext, WORD_SIZE};

#[sel4_cfg(not(KERNEL_MCS))]
use crate::Result;

/// The raw bits of a capability pointer.
pub type CPtrBits = sys::seL4_CPtr;

/// A capability pointer.
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

/// A capability pointer with a number of bits to resolve.
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

/// A capability pointer in the current CSpace.
///
/// - The `T` parameter is a [`CapType`] marking the type of the pointed-to capability.
/// - The `C` parameter is a strategy for discovering the current thread's IPC buffer. When the
///   `"state"` feature is enabled, [`NoExplicitInvocationContext`] is an alias for
///   [`ImplicitInvocationContext`](crate::ImplicitInvocationContext), which uses the [`IPCBuffer`]
///   set by [`set_ipc_buffer`](crate::set_ipc_buffer). Otherwise, it is an alias for
///   [`NoInvocationContext`](crate::NoInvocationContext), which does not implement
///   [`InvocationContext`]. In such cases, the [`with`](LocalCPtr::with) method is used to specify
///   an invocation context before the capability is invoked.
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
    //! Markers corresponding to capability types and classes of capability types.
    //!
    //! These types are used for marking [`LocalCPtr`](crate::LocalCPtr).

    use sel4_config::sel4_cfg_if;

    use crate::declare_cap_type;

    pub use crate::arch::cap_type_arch::*;

    declare_cap_type! {
        /// Corresponds to `seL4_Untyped`.
        Untyped
    }

    declare_cap_type! {
        /// Corresponds to the endpoint capability type.
        Endpoint
    }

    declare_cap_type! {
        /// Corresponds to the notification capability type.
        Notification
    }

    declare_cap_type! {
        /// Corresponds to `seL4_TCB`.
        TCB
    }

    declare_cap_type! {
        /// Corresponds to `seL4_CNode`.
        CNode
    }

    declare_cap_type! {
        /// Corresponds to `seL4_IRQControl`.
        IRQControl
    }

    declare_cap_type! {
        /// Corresponds to `seL4_IRQHandler`.
        IRQHandler
    }

    declare_cap_type! {
        /// Corresponds to `seL4_ASIDControl`.
        ASIDControl
    }

    declare_cap_type! {
        /// Corresponds to `seL4_ASIDPool`.
        ASIDPool
    }

    declare_cap_type! {
        /// Corresponds to the null capability.
        Null
    }

    declare_cap_type! {
        /// Any capability.
        Unspecified
    }

    sel4_cfg_if! {
        if #[cfg(KERNEL_MCS)] {
            declare_cap_type! {
                /// Corresponds to the reply capability type (MCS only).
                Reply
            }

            declare_cap_type! {
                /// Corresponds to the scheduling context capability type (MCS only).
                SchedContext
            }
        }
    }
}

use local_cptr::*;

pub mod local_cptr {
    //! Marked aliases of [`LocalCPtr`](crate::LocalCPtr).
    //!
    //! Each type `$t<C = NoExplicitInvocationContext>` in this module is an alias for `LocalCPtr<$t, C>`.

    use sel4_config::sel4_cfg_if;

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

    sel4_cfg_if! {
        if #[cfg(KERNEL_MCS)] {
            declare_local_cptr_alias!(Reply);
            declare_local_cptr_alias!(SchedContext);
        }
    }
}

impl<C> Unspecified<C> {
    pub fn downcast<T: CapType>(self) -> LocalCPtr<T, C> {
        self.cast()
    }
}

/// A [`CPtrWithDepth`] in a particular [`CNode`].
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct AbsoluteCPtr<C = NoExplicitInvocationContext> {
    root: CNode<C>,
    path: CPtrWithDepth,
}

impl<C> AbsoluteCPtr<C> {
    pub const fn root(&self) -> &CNode<C> {
        &self.root
    }

    pub const fn path(&self) -> &CPtrWithDepth {
        &self.path
    }

    pub fn with<C1>(self, context: C1) -> AbsoluteCPtr<C1> {
        AbsoluteCPtr {
            root: self.root.with(context),
            path: self.path,
        }
    }

    pub fn without_context(self) -> AbsoluteCPtr {
        self.with(NoExplicitInvocationContext::new())
    }
}

impl<C: InvocationContext> AbsoluteCPtr<C> {
    pub fn invoke<R>(self, f: impl FnOnce(CPtr, CPtrWithDepth, &mut IPCBuffer) -> R) -> R {
        let path = *self.path();
        self.root
            .invoke(|cptr, ipc_buffer| f(cptr, path, ipc_buffer))
    }
}

/// Trait for types whose members which logically contain a [`CPtrWithDepth`].
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
    pub fn relative<T: HasCPtrWithDepth>(self, path: T) -> AbsoluteCPtr<C> {
        AbsoluteCPtr {
            root: self,
            path: path.cptr_with_depth(),
        }
    }

    pub fn relative_bits_with_depth(self, bits: CPtrBits, depth: usize) -> AbsoluteCPtr<C> {
        self.relative(CPtrWithDepth::from_bits_with_depth(bits, depth))
    }

    pub fn relative_self(self) -> AbsoluteCPtr<C> {
        self.relative(CPtrWithDepth::empty())
    }
}

impl<C: InvocationContext> CNode<C> {
    #[sel4_cfg(not(KERNEL_MCS))]
    pub fn save_caller(self, ep: Endpoint) -> Result<()> {
        self.relative(ep).save_caller()
    }
}
