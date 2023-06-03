use core::mem::size_of_val;

#[cfg(feature = "alloc")]
use alloc::string::String;

use crate::frame_init::*;
use crate::indirect::*;
use crate::object_name::*;
use crate::spec::*;

pub trait Footprint {
    fn external_footprint(&self) -> usize {
        0
    }

    fn total_footprint(&self) -> usize {
        size_of_val(self) + self.external_footprint()
    }
}

impl Footprint for IRQEntry {}
impl Footprint for ASIDSlotEntry {}
impl Footprint for Cap {}
impl Footprint for CapTableEntry {}
impl Footprint for Word {}
impl Footprint for IndirectBytesContent {}
impl Footprint for IndirectObjectName {}

#[cfg(feature = "deflate")]
impl Footprint for IndirectDeflatedBytesContent {}

impl<'a, T: Sized + Footprint> Footprint for Indirect<'a, T> {
    fn external_footprint(&self) -> usize {
        self.inner().total_footprint()
    }
}

impl<T: Footprint> Footprint for Option<T> {
    fn external_footprint(&self) -> usize {
        match self {
            Some(val) => val.external_footprint(),
            None => 0,
        }
    }
}

impl<'a, T: Footprint> Footprint for Indirect<'a, [T]> {
    fn external_footprint(&self) -> usize {
        self.inner().iter().map(Footprint::total_footprint).sum()
    }
}

impl<'a, N: Footprint, F: Footprint> Footprint for Spec<'a, N, F> {
    fn external_footprint(&self) -> usize {
        self.objects.external_footprint()
            + self.irqs.external_footprint()
            + self.asid_slots.external_footprint()
    }
}

impl<'a, N: Footprint, F: Footprint> Footprint for NamedObject<'a, N, F> {
    fn external_footprint(&self) -> usize {
        self.name.external_footprint() + self.object.external_footprint()
    }
}

impl<'a, F: Footprint> Footprint for Object<'a, F> {
    fn external_footprint(&self) -> usize {
        match self {
            Self::CNode(obj) => obj.slots.external_footprint(),
            Self::TCB(obj) => obj.slots.external_footprint() + obj.extra.gprs.external_footprint(),
            Self::IRQ(obj) => obj.slots.external_footprint(),
            Self::Frame(obj) => obj.fill.external_footprint(),
            Self::PageTable(obj) => obj.slots.external_footprint(),
            Self::ArmIRQ(obj) => obj.slots.external_footprint(),
            _ => 0,
        }
    }
}

impl<F: Footprint> Footprint for FillEntry<F> {
    fn external_footprint(&self) -> usize {
        self.content.external_footprint()
    }
}

impl<F: Footprint> Footprint for FillEntryContent<F> {
    fn external_footprint(&self) -> usize {
        match self {
            Self::Data(data) => data.external_footprint(),
            _ => 0,
        }
    }
}

#[cfg(feature = "alloc")]
impl Footprint for String {
    fn external_footprint(&self) -> usize {
        self.len()
    }
}
