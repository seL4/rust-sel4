//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::mem::size_of_val;

#[cfg(all(feature = "alloc", not(feature = "std")))]
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

impl Footprint for IrqEntry {}
impl Footprint for AsidSlotEntry {}
impl Footprint for Cap {}
impl Footprint for CapTableEntry {}
impl Footprint for Word {}
impl Footprint for IndirectBytesContent {}
impl Footprint for IndirectObjectName {}
impl Footprint for IndirectEmbeddedFrame {}

#[cfg(feature = "deflate")]
impl Footprint for IndirectDeflatedBytesContent {}

impl<T: Sized + Footprint> Footprint for Indirect<'_, T> {
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

impl<T: Footprint> Footprint for Indirect<'_, [T]> {
    fn external_footprint(&self) -> usize {
        self.inner().iter().map(Footprint::total_footprint).sum()
    }
}

impl<N: Footprint, D: Footprint, M: Footprint> Footprint for Spec<'_, N, D, M> {
    fn external_footprint(&self) -> usize {
        self.objects.external_footprint()
            + self.irqs.external_footprint()
            + self.asid_slots.external_footprint()
    }
}

impl<N: Footprint, D: Footprint, M: Footprint> Footprint for NamedObject<'_, N, D, M> {
    fn external_footprint(&self) -> usize {
        self.name.external_footprint() + self.object.external_footprint()
    }
}

impl<D: Footprint, M: Footprint> Footprint for Object<'_, D, M> {
    fn external_footprint(&self) -> usize {
        match self {
            Self::CNode(obj) => obj.slots.external_footprint(),
            Self::Tcb(obj) => obj.slots.external_footprint() + obj.extra.gprs.external_footprint(),
            Self::Irq(obj) => obj.slots.external_footprint(),
            Self::Frame(obj) => obj.init.external_footprint(),
            Self::PageTable(obj) => obj.slots.external_footprint(),
            Self::ArmIrq(obj) => obj.slots.external_footprint(),
            _ => 0,
        }
    }
}

impl<D: Footprint, M: Footprint> Footprint for FrameInit<'_, D, M> {
    fn external_footprint(&self) -> usize {
        match self {
            Self::Fill(fill) => fill.external_footprint(),
            Self::Embedded(embedded) => embedded.external_footprint(),
        }
    }
}

impl<D: Footprint> Footprint for Fill<'_, D> {
    fn external_footprint(&self) -> usize {
        self.entries.external_footprint()
    }
}

impl<D: Footprint> Footprint for FillEntry<D> {
    fn external_footprint(&self) -> usize {
        self.content.external_footprint()
    }
}

impl<D: Footprint> Footprint for FillEntryContent<D> {
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
