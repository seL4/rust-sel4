use core::fmt;
use core::marker::PhantomData;

use crate::{sys, Result, WORD_SIZE};

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

    // for when depth == 0
    pub const fn arbitrary() -> Self {
        Self::from_bits(0)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LocalCPtr<T: CapType> {
    phantom: PhantomData<T>,
    cptr: CPtr,
}

impl<T: CapType> LocalCPtr<T> {
    pub const fn cptr(self) -> CPtr {
        self.cptr
    }

    pub const fn from_cptr(cptr: CPtr) -> Self {
        Self {
            phantom: PhantomData,
            cptr,
        }
    }

    pub const fn bits(self) -> CPtrBits {
        self.cptr().bits()
    }

    pub const fn from_bits(bits: CPtrBits) -> Self {
        CPtr::from_bits(bits).cast()
    }
}

impl<T: CapType> fmt::Debug for LocalCPtr<T> {
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

    #[derive(Copy, Clone, Eq, PartialEq, CapType)]
    pub struct Untyped;
    #[derive(Copy, Clone, Eq, PartialEq, CapType)]
    pub struct Endpoint;
    #[derive(Copy, Clone, Eq, PartialEq, CapType)]
    pub struct Notification;
    #[derive(Copy, Clone, Eq, PartialEq, CapType)]
    pub struct TCB;
    #[derive(Copy, Clone, Eq, PartialEq, CapType)]
    pub struct VCPU;
    #[derive(Copy, Clone, Eq, PartialEq, CapType)]
    pub struct CNode;
    #[derive(Copy, Clone, Eq, PartialEq, CapType)]
    pub struct SmallPage;
    #[derive(Copy, Clone, Eq, PartialEq, CapType)]
    pub struct LargePage;
    #[derive(Copy, Clone, Eq, PartialEq, CapType)]
    pub struct HugePage;
    #[derive(Copy, Clone, Eq, PartialEq, CapType)]
    pub struct PGD;
    #[derive(Copy, Clone, Eq, PartialEq, CapType)]
    pub struct PUD;
    #[derive(Copy, Clone, Eq, PartialEq, CapType)]
    pub struct PD;
    #[derive(Copy, Clone, Eq, PartialEq, CapType)]
    pub struct PT;
    #[derive(Copy, Clone, Eq, PartialEq, CapType)]
    pub struct IRQControl;
    #[derive(Copy, Clone, Eq, PartialEq, CapType)]
    pub struct IRQHandler;
    #[derive(Copy, Clone, Eq, PartialEq, CapType)]
    pub struct ASIDControl;
    #[derive(Copy, Clone, Eq, PartialEq, CapType)]
    pub struct ASIDPool;
    #[derive(Copy, Clone, Eq, PartialEq, CapType)]
    pub struct Unspecified;
    #[derive(Copy, Clone, Eq, PartialEq, CapType)]
    pub struct Null;
}

use local_cptr::*;

pub mod local_cptr {
    use super::{cap_type, LocalCPtr};

    pub type Untyped = LocalCPtr<cap_type::Untyped>;
    pub type Endpoint = LocalCPtr<cap_type::Endpoint>;
    pub type Notification = LocalCPtr<cap_type::Notification>;
    pub type TCB = LocalCPtr<cap_type::TCB>;
    pub type VCPU = LocalCPtr<cap_type::VCPU>;
    pub type CNode = LocalCPtr<cap_type::CNode>;
    pub type SmallPage = LocalCPtr<cap_type::SmallPage>;
    pub type LargePage = LocalCPtr<cap_type::LargePage>;
    pub type HugePage = LocalCPtr<cap_type::HugePage>;
    pub type PGD = LocalCPtr<cap_type::PGD>;
    pub type PUD = LocalCPtr<cap_type::PUD>;
    pub type PD = LocalCPtr<cap_type::PD>;
    pub type PT = LocalCPtr<cap_type::PT>;
    pub type IRQControl = LocalCPtr<cap_type::IRQControl>;
    pub type IRQHandler = LocalCPtr<cap_type::IRQHandler>;
    pub type ASIDControl = LocalCPtr<cap_type::ASIDControl>;
    pub type ASIDPool = LocalCPtr<cap_type::ASIDPool>;
    pub type Unspecified = LocalCPtr<cap_type::Unspecified>;
    pub type Null = LocalCPtr<cap_type::Null>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelativeCPtr {
    root: CNode,
    path: CPtrWithDepth,
}

impl RelativeCPtr {
    pub const fn root(&self) -> CNode {
        self.root
    }

    pub const fn path(&self) -> &CPtrWithDepth {
        &self.path
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

impl<T: CapType> HasCPtrWithDepth for LocalCPtr<T> {
    fn cptr_with_depth(self) -> CPtrWithDepth {
        self.cptr().into()
    }
}

impl HasCPtrWithDepth for CPtrWithDepth {
    fn cptr_with_depth(self) -> CPtrWithDepth {
        self
    }
}

impl CNode {
    pub fn relative<T: HasCPtrWithDepth>(self, path: T) -> RelativeCPtr {
        RelativeCPtr {
            root: self,
            path: path.cptr_with_depth(),
        }
    }

    pub fn relative_bits_with_depth(self, bits: CPtrBits, depth: usize) -> RelativeCPtr {
        self.relative(CPtrWithDepth::from_bits_with_depth(bits, depth))
    }

    pub fn relative_self(self) -> RelativeCPtr {
        // TODO which is preferred?
        // self.relative(CPtr::arbitrary)
        self.relative(self)
    }

    pub fn save_caller(&self, ep: Endpoint) -> Result<()> {
        self.relative(ep).save_caller()
    }
}

impl Unspecified {
    pub fn downcast<T: CapType>(self) -> LocalCPtr<T> {
        LocalCPtr::from_cptr(self.cptr())
    }
}

// HACK until we get negative reasoning
pub auto trait NotCNodeCapType {}

impl !NotCNodeCapType for cap_type::CNode {}
