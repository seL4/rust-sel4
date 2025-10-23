//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4::{AbsoluteCPtr, init_thread::Slot};

use crate::{CSlotAllocator, CapDLInitializerError};

const NUM_SLOTS: usize = 2;

pub(crate) struct HoldSlots<T> {
    slots: [Slot; NUM_SLOTS],
    slots_occupied: [bool; NUM_SLOTS],
    which_slot: usize,
    absolute_cptr_of: T,
}

impl<T> HoldSlots<T> {
    pub(crate) fn new(
        cslot_allocator: &mut CSlotAllocator,
        absolute_cptr_of: T,
    ) -> Result<Self, CapDLInitializerError> {
        Ok(Self {
            slots: {
                let mut f = || cslot_allocator.alloc();
                [f()?, f()?]
            },
            slots_occupied: [false; NUM_SLOTS],
            which_slot: 0,
            absolute_cptr_of,
        })
    }
}

impl<T: FnMut(Slot) -> AbsoluteCPtr> HoldSlots<T> {
    pub(crate) fn get_slot(&mut self) -> Result<Slot, CapDLInitializerError> {
        if self.slots_occupied[self.which_slot] {
            (self.absolute_cptr_of)(self.slots[self.which_slot]).delete()?;
            self.slots_occupied[self.which_slot] = false;
        }
        Ok(self.slots[self.which_slot])
    }

    pub(crate) fn report_used(&mut self) {
        self.slots_occupied[self.which_slot] = true;
        self.which_slot = (self.which_slot + 1) % NUM_SLOTS;
    }
}
