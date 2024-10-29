//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;
use core::ops::Range;

use cfg_if::cfg_if;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use sel4_capdl_initializer_types_derive::{IsCap, IsObject, IsObjectWithCapTable};

use crate::{FrameInit, HasCapTable, Indirect};

cfg_if! {
    if #[cfg(feature = "sel4")] {
        pub use sel4::Word;
    } else {
        pub type Word = u64;
    }
}

pub type Badge = Word;
pub type CPtr = Word;

pub type ObjectId = usize;

pub type CapSlot = usize;
pub type CapTableEntry = (CapSlot, Cap);

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Spec<'a, N, D, M> {
    pub objects: Indirect<'a, [NamedObject<'a, N, D, M>]>,
    pub irqs: Indirect<'a, [IrqEntry]>,
    pub asid_slots: Indirect<'a, [AsidSlotEntry]>,
    pub root_objects: Range<ObjectId>,
    pub untyped_covers: Indirect<'a, [UntypedCover]>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IrqEntry {
    pub irq: Word,
    pub handler: ObjectId,
}

pub type AsidSlotEntry = ObjectId;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UntypedCover {
    pub parent: ObjectId,
    pub children: Range<ObjectId>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NamedObject<'a, N, D, M> {
    pub name: N,
    pub object: Object<'a, D, M>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Object<'a, D, M> {
    Untyped(object::Untyped),
    Endpoint,
    Notification,
    CNode(object::CNode<'a>),
    Tcb(object::Tcb<'a>),
    Irq(object::Irq<'a>),
    VCpu,
    Frame(object::Frame<'a, D, M>),
    PageTable(object::PageTable<'a>),
    AsidPool(object::AsidPool),
    ArmIrq(object::ArmIrq<'a>),
    IrqMsi(object::IrqMsi<'a>),
    IrqIOApic(object::IrqIOApic<'a>),
    IOPorts(object::IOPorts),
    SchedContext(object::SchedContext),
    Reply,
}

impl<D, M> Object<'_, D, M> {
    pub fn paddr(&self) -> Option<usize> {
        match self {
            Object::Untyped(obj) => obj.paddr,
            Object::Frame(obj) => obj.paddr,
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Cap {
    Untyped(cap::Untyped),
    Endpoint(cap::Endpoint),
    Notification(cap::Notification),
    CNode(cap::CNode),
    Tcb(cap::Tcb),
    IrqHandler(cap::IrqHandler),
    VCpu(cap::VCpu),
    Frame(cap::Frame),
    PageTable(cap::PageTable),
    AsidPool(cap::AsidPool),
    ArmIrqHandler(cap::ArmIrqHandler),
    IrqMsiHandler(cap::IrqMsiHandler),
    IrqIOApicHandler(cap::IrqIOApicHandler),
    IOPorts(cap::IOPorts),
    SchedContext(cap::SchedContext),
    Reply(cap::Reply),
}

impl Cap {
    pub fn obj(&self) -> ObjectId {
        match self {
            Cap::Untyped(cap) => cap.object,
            Cap::Endpoint(cap) => cap.object,
            Cap::Notification(cap) => cap.object,
            Cap::CNode(cap) => cap.object,
            Cap::Frame(cap) => cap.object,
            Cap::Tcb(cap) => cap.object,
            Cap::IrqHandler(cap) => cap.object,
            Cap::VCpu(cap) => cap.object,
            Cap::PageTable(cap) => cap.object,
            Cap::AsidPool(cap) => cap.object,
            Cap::ArmIrqHandler(cap) => cap.object,
            Cap::IrqMsiHandler(cap) => cap.object,
            Cap::IrqIOApicHandler(cap) => cap.object,
            Cap::IOPorts(cap) => cap.object,
            Cap::SchedContext(cap) => cap.object,
            Cap::Reply(cap) => cap.object,
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
    pub struct Tcb<'a> {
        pub slots: Indirect<'a, [CapTableEntry]>,
        pub extra: Indirect<'a, TcbExtraInfo<'a>>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct TcbExtraInfo<'a> {
        pub ipc_buffer_addr: Word,

        pub affinity: Word,
        pub prio: u8,
        pub max_prio: u8,
        pub resume: bool,

        pub ip: Word,
        pub sp: Word,
        pub gprs: Indirect<'a, [Word]>,

        pub master_fault_ep: Option<CPtr>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Irq<'a> {
        pub slots: Indirect<'a, [CapTableEntry]>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Frame<'a, D, M> {
        pub size_bits: usize,
        pub paddr: Option<usize>,
        pub init: FrameInit<'a, D, M>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct PageTable<'a> {
        pub is_root: bool,
        pub level: Option<u8>,
        pub slots: Indirect<'a, [CapTableEntry]>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct AsidPool {
        pub high: Word,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct ArmIrq<'a> {
        pub slots: Indirect<'a, [CapTableEntry]>,
        pub extra: Indirect<'a, ArmIrqExtraInfo>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct ArmIrqExtraInfo {
        pub trigger: Word,
        pub target: Word,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct IrqMsi<'a> {
        pub slots: Indirect<'a, [CapTableEntry]>,
        pub extra: Indirect<'a, IrqMsiExtraInfo>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct IrqMsiExtraInfo {
        pub handle: Word,
        pub pci_bus: Word,
        pub pci_dev: Word,
        pub pci_func: Word,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct IrqIOApic<'a> {
        pub slots: Indirect<'a, [CapTableEntry]>,
        pub extra: Indirect<'a, IrqIOApicExtraInfo>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct IrqIOApicExtraInfo {
        pub ioapic: Word,
        pub pin: Word,
        pub level: Word,
        pub polarity: Word,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct IOPorts {
        pub start_port: Word,
        pub end_port: Word,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct SchedContext {
        pub size_bits: usize,
        pub extra: SchedContextExtraInfo,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct SchedContextExtraInfo {
        pub period: u64,
        pub budget: u64,
        pub badge: Badge,
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
        // TODO
        //   parse-capDL uses badge=0 to mean no badge. Is that good
        //   enough, or do we ever need to actually use the badge value '0'?
        // TODO
        //   Is it correct that these are ignored in the case of Tcb::SLOT_TEMP_FAULT_EP?
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
    pub struct Tcb {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct IrqHandler {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct VCpu {
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
    pub struct PageTable {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct AsidPool {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct ArmIrqHandler {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct IrqMsiHandler {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct IrqIOApicHandler {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct IOPorts {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct SchedContext {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Reply {
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
