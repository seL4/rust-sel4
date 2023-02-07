use core::fmt;
use core::ops::Range;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use capdl_types_derive::{IsCap, IsObject, IsObjectWithCapTable};

use crate::{HasCapTable, Indirect};

// TODO
// Prepare for broader platform support:
// - Eliminate use of `usize`.
// - Parameterize with token `Arch` type?
// - Use generic `Frame` object variant with `size_bits` field.

pub type Word = u64;
pub type Badge = Word;
pub type CPtr = Word;

pub type ObjectId = usize;

pub type CapSlot = usize;
pub type CapTableEntry = (CapSlot, Cap);

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Spec<'a, F, N> {
    pub objects: Indirect<'a, [NamedObject<'a, F, N>]>,
    pub irqs: Indirect<'a, [IRQEntry]>,
    pub asid_slots: Indirect<'a, [ASIDSlotEntry]>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IRQEntry {
    pub irq: Word,
    pub handler: ObjectId,
}

pub type ASIDSlotEntry = ObjectId;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NamedObject<'a, N, F> {
    pub name: N,
    pub object: Object<'a, F>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Object<'a, F> {
    Untyped(object::Untyped),
    Endpoint,
    Notification,
    CNode(object::CNode<'a>),
    TCB(object::TCB<'a>),
    IRQ(object::IRQ<'a>),
    VCPU,
    Frame(object::Frame<'a, F>),
    PT(object::PT<'a>),
    PD(object::PD<'a>),
    PUD(object::PUD<'a>),
    PGD(object::PGD<'a>),
    ASIDPool(object::ASIDPool),
    ArmIRQ(object::ArmIRQ<'a>),
}

impl<'a, F> Object<'a, F> {
    pub fn paddr(&self) -> Option<usize> {
        match self {
            Object::Untyped(obj) => obj.paddr,
            Object::Frame(obj) => obj.paddr,
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
    Frame(cap::Frame),
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
            Cap::Frame(cap) => cap.object,
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

// TODO Would packing have an actual effect on memory footprint?
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FillEntryContentBootInfo {
    pub id: FillEntryContentBootInfoId,
    pub offset: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FillEntryContentBootInfoId {
    Fdt,
}

// // //

pub mod object {
    use super::*;

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Untyped {
        pub size_bits: usize,
        pub paddr: Option<usize>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct CNode<'a> {
        pub size_bits: usize,
        pub slots: Indirect<'a, [CapTableEntry]>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct TCB<'a> {
        pub slots: Indirect<'a, [CapTableEntry]>,
        pub extra: Indirect<'a, TCBExtraInfo<'a>>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct TCBExtraInfo<'a> {
        pub ipc_buffer_addr: Word,
        pub fault_ep: CPtr,

        pub affinity: Word,
        pub prio: u8,
        pub max_prio: u8,
        pub resume: bool,

        pub ip: Word,
        pub sp: Word,
        pub spsr: Word,
        pub gprs: Indirect<'a, [Word]>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct IRQ<'a> {
        pub slots: Indirect<'a, [CapTableEntry]>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Frame<'a, F> {
        pub size_bits: usize,
        pub paddr: Option<usize>,
        pub fill: Indirect<'a, [FillEntry<F>]>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct PT<'a> {
        pub slots: Indirect<'a, [CapTableEntry]>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct PD<'a> {
        pub slots: Indirect<'a, [CapTableEntry]>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct PUD<'a> {
        pub slots: Indirect<'a, [CapTableEntry]>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct PGD<'a> {
        pub slots: Indirect<'a, [CapTableEntry]>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct ASIDPool {
        pub high: Word,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct ArmIRQ<'a> {
        pub slots: Indirect<'a, [CapTableEntry]>,
        pub extra: Indirect<'a, ArmIRQExtraInfo>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct ArmIRQExtraInfo {
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
    pub struct Frame {
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
