use core::fmt;
use core::iter;
use core::slice;

use crate::{cap, object, Cap, CapSlot, CapTableEntry, TryFromCapError};

// NOTE
// Magic constants must be kept in sync with capDL-tool.

pub trait HasCapTable {
    fn slots(&self) -> &[CapTableEntry];

    fn maybe_slot(&self, slot: CapSlot) -> Option<&Cap> {
        self.slots()
            .as_ref()
            .iter()
            .find_map(|(k, v)| if k == &slot { Some(v) } else { None })
    }

    fn maybe_slot_as<'a, T: TryFrom<&'a Cap>>(&'a self, slot: CapSlot) -> Option<T>
    where
        <T as TryFrom<&'a Cap>>::Error: fmt::Debug,
    {
        self.maybe_slot(slot).map(|cap| cap.try_into().unwrap())
    }

    fn slot_as<'a, T: TryFrom<&'a Cap>>(&'a self, slot: CapSlot) -> T
    where
        <T as TryFrom<&'a Cap>>::Error: fmt::Debug,
    {
        self.maybe_slot_as(slot).unwrap()
    }

    #[allow(clippy::type_complexity)]
    fn slots_as<'a, T: TryFrom<&'a Cap>>(
        &'a self,
    ) -> iter::Map<slice::Iter<'_, (usize, Cap)>, fn(&'a (usize, Cap)) -> (usize, T)>
    where
        <T as TryFrom<&'a Cap>>::Error: fmt::Debug,
    {
        self.slots()
            .iter()
            .map(|(k, v)| (*k, T::try_from(v).unwrap()))
    }
}

impl<'a> object::TCB<'a> {
    pub const SLOT_CSPACE: CapSlot = 0;
    pub const SLOT_VSPACE: CapSlot = 1;
    pub const SLOT_IPC_BUFFER: CapSlot = 4;
    pub const SLOT_SC: CapSlot = 6;
    pub const SLOT_TEMP_FAULT_EP: CapSlot = 7;
    pub const SLOT_BOUND_NOTIFICATION: CapSlot = 8;
    pub const SLOT_VCPU: CapSlot = 9;

    pub fn cspace(&self) -> &cap::CNode {
        self.slot_as(Self::SLOT_CSPACE)
    }

    pub fn vspace(&self) -> &cap::PageTable {
        self.slot_as(Self::SLOT_VSPACE)
    }

    pub fn ipc_buffer(&self) -> &cap::Frame {
        self.slot_as(Self::SLOT_IPC_BUFFER)
    }

    pub fn sc(&self) -> Option<&cap::SchedContext> {
        self.maybe_slot_as(Self::SLOT_SC)
    }

    pub fn temp_fault_ep(&self) -> Option<&cap::Endpoint> {
        self.maybe_slot_as(Self::SLOT_TEMP_FAULT_EP)
    }

    pub fn bound_notification(&self) -> Option<&cap::Notification> {
        self.maybe_slot_as(Self::SLOT_BOUND_NOTIFICATION)
    }

    pub fn vcpu(&self) -> Option<&cap::VCPU> {
        self.maybe_slot_as(Self::SLOT_VCPU)
    }
}

impl<'a> object::IRQ<'a> {
    pub const SLOT_NOTIFICATION: CapSlot = 0;

    pub fn notification(&self) -> Option<&cap::Notification> {
        self.maybe_slot_as(Self::SLOT_NOTIFICATION)
    }
}

impl<'a> object::ArmIRQ<'a> {
    pub const SLOT_NOTIFICATION: CapSlot = 0;

    pub fn notification(&self) -> Option<&cap::Notification> {
        self.maybe_slot_as(Self::SLOT_NOTIFICATION)
    }
}

// // //

impl<'a> object::PageTable<'a> {
    pub fn entries(&self) -> impl Iterator<Item = (CapSlot, PageTableEntry)> {
        self.slots_as()
    }

    pub fn frames(&self) -> impl Iterator<Item = (CapSlot, &cap::Frame)> {
        self.slots_as()
    }

    pub fn page_tables(&self) -> impl Iterator<Item = (CapSlot, &cap::PageTable)> {
        self.slots_as()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PageTableEntry<'a> {
    PageTable(&'a cap::PageTable),
    Frame(&'a cap::Frame),
}

impl<'a> PageTableEntry<'a> {
    pub fn page_table(&self) -> Option<&'a cap::PageTable> {
        match self {
            Self::PageTable(cap) => Some(cap),
            _ => None,
        }
    }

    pub fn frame(&self) -> Option<&'a cap::Frame> {
        match self {
            Self::Frame(cap) => Some(cap),
            _ => None,
        }
    }
}

impl<'a> TryFrom<&'a Cap> for PageTableEntry<'a> {
    type Error = TryFromCapError;

    fn try_from(cap: &'a Cap) -> Result<Self, Self::Error> {
        Ok(match cap {
            Cap::PageTable(cap) => PageTableEntry::PageTable(cap),
            Cap::Frame(cap) => PageTableEntry::Frame(cap),
            _ => return Err(TryFromCapError),
        })
    }
}
