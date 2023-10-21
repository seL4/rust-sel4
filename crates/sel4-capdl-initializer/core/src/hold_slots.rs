//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::array;

use sel4::{AbsoluteCPtr, InitCSpaceSlot};

use crate::{CSlotAllocator, CapDLInitializerError};

const NUM_SLOTS: usize = 2;

pub(crate) struct HoldSlots<T> {
    slots: [InitCSpaceSlot; NUM_SLOTS],
    slots_occupied: [bool; NUM_SLOTS],
    which_slot: usize,
    relative_cptr_of: T,
}

impl<T> HoldSlots<T> {
    pub(crate) fn new(
        cslot_allocator: &mut CSlotAllocator,
        relative_cptr_of: T,
    ) -> Result<Self, CapDLInitializerError> {
        Ok(Self {
            slots: array::try_from_fn(|_| cslot_allocator.alloc())?,
            slots_occupied: [false; NUM_SLOTS],
            which_slot: 0,
            relative_cptr_of,
        })
    }
}

impl<T: FnMut(InitCSpaceSlot) -> AbsoluteCPtr> HoldSlots<T> {
    pub(crate) fn get_slot(&mut self) -> Result<InitCSpaceSlot, CapDLInitializerError> {
        if self.slots_occupied[self.which_slot] {
            (self.relative_cptr_of)(self.slots[self.which_slot]).delete()?;
            self.slots_occupied[self.which_slot] = false;
        }
        Ok(self.slots[self.which_slot])
    }

    pub(crate) fn report_used(&mut self) {
        self.slots_occupied[self.which_slot] = true;
        self.which_slot = (self.which_slot + 1) % NUM_SLOTS;
    }
}
