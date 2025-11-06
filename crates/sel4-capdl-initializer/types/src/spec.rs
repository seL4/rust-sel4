//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::ops::Range;

use rkyv::Archive;
use rkyv::option::ArchivedOption;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use sel4_capdl_initializer_types_derive::{HasCapTable, IsCap, IsObject};

use crate::{HasArchivedCapTable, HasCapTable};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
#[rkyv(derive(Debug, Copy, Clone, Eq, PartialEq))]
pub struct ObjectId(pub u32);

impl From<usize> for ObjectId {
    fn from(x: usize) -> ObjectId {
        ObjectId(x.try_into().unwrap())
    }
}

impl From<usize> for ArchivedObjectId {
    fn from(x: usize) -> ArchivedObjectId {
        ArchivedObjectId(x.try_into().unwrap())
    }
}

impl From<ObjectId> for usize {
    fn from(x: ObjectId) -> usize {
        x.0.try_into().unwrap()
    }
}

impl From<ArchivedObjectId> for usize {
    fn from(x: ArchivedObjectId) -> usize {
        x.0.try_into().unwrap()
    }
}

impl ObjectId {
    pub fn into_usize_range(range: &Range<ObjectId>) -> Range<usize> {
        range.start.into()..range.end.into()
    }
}

impl ArchivedObjectId {
    pub fn into_usize_range(range: &Range<ArchivedObjectId>) -> Range<usize> {
        range.start.into()..range.end.into()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
#[rkyv(derive(Debug, Copy, Clone, Eq, PartialEq))]
pub struct CapSlot(pub u32);

impl From<usize> for CapSlot {
    fn from(x: usize) -> CapSlot {
        CapSlot(x.try_into().unwrap())
    }
}

impl From<usize> for ArchivedCapSlot {
    fn from(x: usize) -> ArchivedCapSlot {
        ArchivedCapSlot(x.try_into().unwrap())
    }
}

impl From<CapSlot> for usize {
    fn from(x: CapSlot) -> usize {
        x.0.try_into().unwrap()
    }
}

impl From<ArchivedCapSlot> for usize {
    fn from(x: ArchivedCapSlot) -> usize {
        x.0.try_into().unwrap()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct CapTableEntry {
    pub slot: CapSlot,
    pub cap: Cap,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct Spec<D> {
    pub objects: Vec<NamedObject<D>>,
    pub irqs: Vec<IrqEntry>,
    pub asid_slots: Vec<AsidSlotEntry>,
    pub root_objects: Range<ObjectId>,
    pub untyped_covers: Vec<UntypedCover>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
#[rkyv(derive(Debug, Copy, Clone, Eq, PartialEq))]
pub struct Word(pub u64);

impl From<u64> for Word {
    fn from(x: u64) -> Word {
        Word(x)
    }
}

impl From<Word> for u64 {
    fn from(x: Word) -> u64 {
        x.0
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct IrqEntry {
    pub irq: Word,
    pub handler: ObjectId,
}

pub type AsidSlotEntry = ObjectId;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct UntypedCover {
    pub parent: ObjectId,
    pub children: Range<ObjectId>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct NamedObject<D> {
    pub name: Option<String>,
    pub object: Object<D>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub enum Object<D> {
    Untyped(object::Untyped),
    Endpoint,
    Notification,
    CNode(object::CNode),
    Tcb(object::Tcb),
    Irq(object::Irq),
    VCpu,
    Frame(object::Frame<D>),
    PageTable(object::PageTable),
    AsidPool(object::AsidPool),
    ArmIrq(object::ArmIrq),
    IrqMsi(object::IrqMsi),
    IrqIOApic(object::IrqIOApic),
    RiscvIrq(object::RiscvIrq),
    IOPorts(object::IOPorts),
    SchedContext(object::SchedContext),
    Reply,
    ArmSmc,
}

pub trait IsObject<D> {
    fn into_object(self) -> Object<D>;
    fn try_from_object(obj: &Object<D>) -> Option<&Self>;
}

impl<D> Object<D> {
    pub fn as_<T: IsObject<D>>(&self) -> Option<&T> {
        T::try_from_object(self)
    }

    pub fn paddr(&self) -> Option<Word> {
        match self {
            Self::Untyped(obj) => obj.paddr,
            Self::Frame(obj) => obj.paddr,
            _ => None,
        }
    }

    pub fn slots(&self) -> Option<&[CapTableEntry]> {
        Some(match self {
            Self::CNode(obj) => obj.slots(),
            Self::Tcb(obj) => obj.slots(),
            Self::PageTable(obj) => obj.slots(),
            Self::ArmIrq(obj) => obj.slots(),
            Self::IrqMsi(obj) => obj.slots(),
            Self::IrqIOApic(obj) => obj.slots(),
            Self::RiscvIrq(obj) => obj.slots(),
            _ => return None,
        })
    }

    pub fn slots_mut(&mut self) -> Option<&mut Vec<CapTableEntry>> {
        Some(match self {
            Self::CNode(obj) => &mut obj.slots,
            Self::Tcb(obj) => &mut obj.slots,
            Self::PageTable(obj) => &mut obj.slots,
            Self::ArmIrq(obj) => &mut obj.slots,
            Self::IrqMsi(obj) => &mut obj.slots,
            Self::IrqIOApic(obj) => &mut obj.slots,
            Self::RiscvIrq(obj) => &mut obj.slots,
            _ => return None,
        })
    }
}

pub trait IsArchivedObject<D: Archive>: Sized {
    fn try_from_object(obj: &ArchivedObject<D>) -> Option<&Self>;
}

impl<D: Archive> ArchivedObject<D> {
    pub fn as_<T: IsArchivedObject<D>>(&self) -> Option<&T> {
        T::try_from_object(self)
    }

    pub fn paddr(&self) -> ArchivedOption<ArchivedWord> {
        match self {
            Self::Untyped(obj) => obj.paddr,
            Self::Frame(obj) => obj.paddr,
            _ => ArchivedOption::None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
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
    RiscvIrqHandler(cap::RiscvIrqHandler),
    IOPorts(cap::IOPorts),
    SchedContext(cap::SchedContext),
    Reply(cap::Reply),
    ArmSmc(cap::ArmSmc),
}

pub trait IsCap {
    fn into_cap(self) -> Cap;
    fn try_from_cap(cap: &Cap) -> Option<&Self>;
}

impl Cap {
    pub fn as_<T: IsCap>(&self) -> Option<&T> {
        T::try_from_cap(self)
    }

    pub fn obj(&self) -> ObjectId {
        match self {
            Self::Untyped(cap) => cap.object,
            Self::Endpoint(cap) => cap.object,
            Self::Notification(cap) => cap.object,
            Self::CNode(cap) => cap.object,
            Self::Frame(cap) => cap.object,
            Self::Tcb(cap) => cap.object,
            Self::IrqHandler(cap) => cap.object,
            Self::VCpu(cap) => cap.object,
            Self::PageTable(cap) => cap.object,
            Self::AsidPool(cap) => cap.object,
            Self::ArmIrqHandler(cap) => cap.object,
            Self::IrqMsiHandler(cap) => cap.object,
            Self::IrqIOApicHandler(cap) => cap.object,
            Self::RiscvIrqHandler(cap) => cap.object,
            Self::IOPorts(cap) => cap.object,
            Self::SchedContext(cap) => cap.object,
            Self::Reply(cap) => cap.object,
            Self::ArmSmc(cap) => cap.object,
        }
    }

    pub fn set_obj(&mut self, object: ObjectId) {
        match self {
            Self::Untyped(cap) => cap.object = object,
            Self::Endpoint(cap) => cap.object = object,
            Self::Notification(cap) => cap.object = object,
            Self::CNode(cap) => cap.object = object,
            Self::Frame(cap) => cap.object = object,
            Self::Tcb(cap) => cap.object = object,
            Self::IrqHandler(cap) => cap.object = object,
            Self::VCpu(cap) => cap.object = object,
            Self::PageTable(cap) => cap.object = object,
            Self::AsidPool(cap) => cap.object = object,
            Self::ArmIrqHandler(cap) => cap.object = object,
            Self::IrqMsiHandler(cap) => cap.object = object,
            Self::IrqIOApicHandler(cap) => cap.object = object,
            Self::RiscvIrqHandler(cap) => cap.object = object,
            Self::IOPorts(cap) => cap.object = object,
            Self::SchedContext(cap) => cap.object = object,
            Self::Reply(cap) => cap.object = object,
            Self::ArmSmc(cap) => cap.object = object,
        }
    }
}

pub trait IsArchivedCap: Sized {
    fn try_from_cap(obj: &ArchivedCap) -> Option<&Self>;
}

impl ArchivedCap {
    pub fn as_<T: IsArchivedCap>(&self) -> Option<&T> {
        T::try_from_cap(self)
    }

    pub fn obj(&self) -> ArchivedObjectId {
        match self {
            Self::Untyped(cap) => cap.object,
            Self::Endpoint(cap) => cap.object,
            Self::Notification(cap) => cap.object,
            Self::CNode(cap) => cap.object,
            Self::Frame(cap) => cap.object,
            Self::Tcb(cap) => cap.object,
            Self::IrqHandler(cap) => cap.object,
            Self::VCpu(cap) => cap.object,
            Self::PageTable(cap) => cap.object,
            Self::AsidPool(cap) => cap.object,
            Self::ArmIrqHandler(cap) => cap.object,
            Self::IrqMsiHandler(cap) => cap.object,
            Self::IrqIOApicHandler(cap) => cap.object,
            Self::RiscvIrqHandler(cap) => cap.object,
            Self::IOPorts(cap) => cap.object,
            Self::SchedContext(cap) => cap.object,
            Self::Reply(cap) => cap.object,
            Self::ArmSmc(cap) => cap.object,
        }
    }
}

pub mod object {
    use super::*;

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct Untyped {
        pub size_bits: u8,
        pub paddr: Option<Word>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, HasCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct CNode {
        pub size_bits: u8,
        pub slots: Vec<CapTableEntry>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, HasCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct Tcb {
        pub slots: Vec<CapTableEntry>,
        pub extra: Box<TcbExtraInfo>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct TcbExtraInfo {
        pub ipc_buffer_addr: Word,

        pub affinity: Word,
        pub prio: u8,
        pub max_prio: u8,
        pub resume: bool,

        pub ip: Word,
        pub sp: Word,
        pub gprs: Vec<Word>,

        pub master_fault_ep: Option<Word>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, HasCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct Irq {
        pub slots: Vec<CapTableEntry>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct Frame<D> {
        pub size_bits: u8,
        pub paddr: Option<Word>,
        pub init: D,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, HasCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct PageTable {
        pub is_root: bool,
        pub level: Option<u8>,
        pub slots: Vec<CapTableEntry>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct AsidPool {
        pub high: Word,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, HasCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct ArmIrq {
        pub slots: Vec<CapTableEntry>,
        pub extra: Box<ArmIrqExtraInfo>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct ArmIrqExtraInfo {
        pub trigger: u8,
        pub target: Word,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, HasCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct IrqMsi {
        pub slots: Vec<CapTableEntry>,
        pub extra: Box<IrqMsiExtraInfo>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct IrqMsiExtraInfo {
        pub handle: Word,
        pub pci_bus: Word,
        pub pci_dev: Word,
        pub pci_func: Word,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, HasCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct IrqIOApic {
        pub slots: Vec<CapTableEntry>,
        pub extra: Box<IrqIOApicExtraInfo>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct IrqIOApicExtraInfo {
        pub ioapic: Word,
        pub pin: Word,
        pub level: Word,
        pub polarity: Word,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, HasCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct RiscvIrq {
        pub slots: Vec<CapTableEntry>,
        pub extra: RiscvIrqExtraInfo,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct RiscvIrqExtraInfo {
        pub trigger: u8,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct IOPorts {
        pub start_port: Word,
        pub end_port: Word,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct SchedContext {
        pub size_bits: u8,
        pub extra: SchedContextExtraInfo,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct SchedContextExtraInfo {
        pub period: u64,
        pub budget: u64,
        pub badge: Word,
    }
}

// TODO Would packing have an actual effect on memory footprint?
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct Rights {
    pub read: bool,
    pub write: bool,
    pub grant: bool,
    pub grant_reply: bool,
}

pub mod cap {
    use super::*;

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct Untyped {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct Endpoint {
        pub object: ObjectId,
        // TODO
        //   parse-capDL uses badge=0 to mean no badge. Is that good
        //   enough, or do we ever need to actually use the badge value '0'?
        // TODO
        //   Is it correct that these are ignored in the case of Tcb::SLOT_TEMP_FAULT_EP?
        pub badge: Word,
        pub rights: Rights,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct Notification {
        pub object: ObjectId,
        pub badge: Word,
        pub rights: Rights,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct CNode {
        pub object: ObjectId,
        pub guard: Word,
        pub guard_size: u8,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct Tcb {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct IrqHandler {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct VCpu {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct Frame {
        pub object: ObjectId,
        pub rights: Rights,
        pub cached: bool,
        pub executable: bool,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct PageTable {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct AsidPool {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct ArmIrqHandler {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct IrqMsiHandler {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct IrqIOApicHandler {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct RiscvIrqHandler {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct IOPorts {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct SchedContext {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct Reply {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct ArmSmc {
        pub object: ObjectId,
    }
}

pub enum PageTableEntry<'a> {
    PageTable(&'a cap::ArchivedPageTable),
    Frame(&'a cap::ArchivedFrame),
}

impl object::ArchivedPageTable {
    pub fn entries(&self) -> impl Iterator<Item = (ArchivedCapSlot, PageTableEntry<'_>)> {
        self.slots.iter().map(|entry| {
            (
                entry.slot,
                match &entry.cap {
                    ArchivedCap::PageTable(cap) => PageTableEntry::PageTable(cap),
                    ArchivedCap::Frame(cap) => PageTableEntry::Frame(cap),
                    _ => panic!(),
                },
            )
        })
    }
}
