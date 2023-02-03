use core::fmt;
use core::ops::Range;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use capdl_types_derive::{IsCap, IsObject};

pub type Word = u64;
pub type Badge = Word;
pub type CPtr = Word;

pub type ObjectId = usize;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Spec<T, U, V> {
    pub objects: T,
    pub irqs: U,
    pub asid_slots: V,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IRQEntry {
    pub irq: Word,
    pub handler: ObjectId,
}

pub type ASIDSlotEntry = ObjectId;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NamedObject<N, C, F> {
    pub name: N,
    pub object: Object<C, F>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Object<C, F> {
    Untyped(object::Untyped),
    Endpoint,
    Notification,
    CNode(object::CNode<C>),
    TCB(object::TCB<C>),
    IRQ(object::IRQ<C>),
    VCPU,
    SmallPage(object::SmallPage<F>),
    LargePage(object::LargePage<F>),
    PT(object::PT<C>),
    PD(object::PD<C>),
    PUD(object::PUD<C>),
    PGD(object::PGD<C>),
    ASIDPool(object::ASIDPool),
    ArmIRQ(object::ArmIRQ<C>),
}

impl<C, F> Object<C, F> {
    pub fn paddr(&self) -> Option<usize> {
        match self {
            Object::Untyped(obj) => obj.paddr,
            Object::SmallPage(obj) => obj.paddr,
            Object::LargePage(obj) => obj.paddr,
            _ => None,
        }
    }
}

// TODO restructure by unifiying .object to outer level?
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Cap {
    Untyped(cap::Untyped),
    Endpoint(cap::Endpoint),
    Notification(cap::Notification),
    CNode(cap::CNode),
    TCB(cap::TCB),
    IRQHandler(cap::IRQHandler),
    VCPU(cap::VCPU),
    SmallPage(cap::SmallPage),
    LargePage(cap::LargePage),
    PT(cap::PT),
    PD(cap::PD),
    PUD(cap::PUD),
    PGD(cap::PGD),
    ASIDPool(cap::ASIDPool),
    ArmIRQHandler(cap::ArmIRQHandler),
}

impl Cap {
    pub fn obj(&self) -> ObjectId {
        match self {
            Cap::Untyped(cap) => cap.object,
            Cap::Endpoint(cap) => cap.object,
            Cap::Notification(cap) => cap.object,
            Cap::CNode(cap) => cap.object,
            Cap::SmallPage(cap) => cap.object,
            Cap::LargePage(cap) => cap.object,
            Cap::TCB(cap) => cap.object,
            Cap::IRQHandler(cap) => cap.object,
            Cap::VCPU(cap) => cap.object,
            Cap::PT(cap) => cap.object,
            Cap::PD(cap) => cap.object,
            Cap::PUD(cap) => cap.object,
            Cap::PGD(cap) => cap.object,
            Cap::ASIDPool(cap) => cap.object,
            Cap::ArmIRQHandler(cap) => cap.object,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rights {
    pub read: bool,
    pub write: bool,
    pub grant: bool,
    pub grant_reply: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FillEntry<F> {
    pub range: Range<usize>,
    pub content: FillEntryContent<F>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FillEntryContent<F> {
    Data(F),
    BootInfo(FillEntryContentBootInfo),
}

impl<F> FillEntryContent<F> {
    pub fn as_data(&self) -> Option<&F> {
        match self {
            Self::Data(data) => Some(&data),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FillEntryContentBootInfo {
    pub id: FillEntryContentBootInfoId,
    pub offset: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FillEntryContentBootInfoId {
    Fdt,
}

pub mod object {
    use super::*;

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Untyped {
        pub size_bits: usize,
        pub paddr: Option<usize>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct CNode<C> {
        pub size_bits: usize,
        pub slots: C,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct TCB<C> {
        pub slots: C,
        pub fault_ep: CPtr,
        pub extra_info: TCBExtraInfo,
        pub init_args: [Option<Word>; TCB_NUM_INIT_ARGS],
    }

    // TODO associate with object::TCB once #[feature(generic_const_exprs)] is complete
    const TCB_NUM_INIT_ARGS: usize = 4;

    #[derive(Debug, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct TCBExtraInfo {
        pub ipc_buffer_addr: Word,

        pub affinity: Word,
        pub prio: u8,
        pub max_prio: u8,
        pub resume: bool,

        pub ip: Word,
        pub sp: Word,
        pub spsr: Word,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct IRQ<C> {
        pub slots: C,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct SmallPage<F> {
        pub paddr: Option<usize>,
        pub fill: F,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct LargePage<F> {
        pub paddr: Option<usize>,
        pub fill: F,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct PT<C> {
        pub slots: C,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct PD<C> {
        pub slots: C,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct PUD<C> {
        pub slots: C,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct PGD<C> {
        pub slots: C,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct ASIDPool {
        pub high: Word,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct ArmIRQ<C> {
        pub slots: C,
        pub trigger: Word,
        pub target: Word,
    }
}

pub mod cap {
    use super::*;

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Untyped {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Endpoint {
        pub object: ObjectId,
        pub badge: Badge,
        pub rights: Rights,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Notification {
        pub object: ObjectId,
        pub badge: Badge,
        pub rights: Rights,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct CNode {
        pub object: ObjectId,
        pub guard: Word,
        pub guard_size: Word,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct TCB {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct IRQHandler {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct VCPU {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct SmallPage {
        pub object: ObjectId,
        pub rights: Rights,
        pub cached: bool,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct LargePage {
        pub object: ObjectId,
        pub rights: Rights,
        pub cached: bool,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct PT {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct PD {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct PUD {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct PGD {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct ASIDPool {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct ArmIRQHandler {
        pub object: ObjectId,
    }
}

// // //

#[derive(Debug)]
pub struct TryFromObjectError;

#[derive(Debug)]
pub struct TryFromCapError;

impl fmt::Display for TryFromObjectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "object type mismatch")
    }
}

impl fmt::Display for TryFromCapError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "object type mismatch")
    }
}
