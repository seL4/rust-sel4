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
    pub const SLOT_BOUND_NOTIFICATION: CapSlot = 8;
    pub const SLOT_VCPU: CapSlot = 9;

    pub fn cspace(&self) -> &cap::CNode {
        self.slot_as(Self::SLOT_CSPACE)
    }

    pub fn vspace(&self) -> &cap::PGD {
        self.slot_as(Self::SLOT_VSPACE)
    }

    pub fn ipc_buffer(&self) -> &cap::Frame {
        self.slot_as(Self::SLOT_IPC_BUFFER)
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

impl<'a> object::PGD<'a> {
    pub fn entries(&self) -> impl Iterator<Item = (CapSlot, &cap::PUD)> {
        self.slots_as()
    }
}

impl<'a> object::PUD<'a> {
    pub fn entries(&self) -> impl Iterator<Item = (CapSlot, &cap::PD)> {
        self.slots_as()
    }
}

impl<'a> object::PD<'a> {
    pub fn entries(&self) -> impl Iterator<Item = (CapSlot, PDEntry)> {
        self.slots_as()
    }
}

impl<'a> object::PT<'a> {
    pub fn entries(&self) -> impl Iterator<Item = (CapSlot, &cap::Frame)> {
        self.slots_as()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PDEntry<'a> {
    PT(&'a cap::PT),
    Frame(&'a cap::Frame),
}

impl<'a> TryFrom<&'a Cap> for PDEntry<'a> {
    type Error = TryFromCapError;

    fn try_from(cap: &'a Cap) -> Result<Self, Self::Error> {
        Ok(match cap {
            Cap::PT(cap) => PDEntry::PT(cap),
            Cap::Frame(cap) => PDEntry::Frame(cap),
            _ => return Err(TryFromCapError),
        })
    }
}
