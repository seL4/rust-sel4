//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

pub struct SlotCountTracker {
    stored_num_free: usize,
}

impl SlotCountTracker {
    pub fn new(initial_num_free: usize) -> Self {
        Self {
            stored_num_free: initial_num_free,
        }
    }

    pub fn report_occupied(&mut self, count: usize) -> Result<(), SlotCountTrackerError> {
        if count > self.stored_num_free {
            return Err(SlotCountTrackerError::ReportOccupiedCountGreaterThanStoredNumFree);
        }
        self.stored_num_free -= count;
        Ok(())
    }

    pub fn redeem_newly_free(
        &mut self,
        current_num_free: usize,
    ) -> Result<usize, SlotCountTrackerError> {
        if current_num_free < self.stored_num_free {
            return Err(SlotCountTrackerError::CurrentNumFreeLessThanStoredNumFree);
        }
        let newly_free = current_num_free - self.stored_num_free;
        self.stored_num_free = current_num_free;
        Ok(newly_free)
    }
}

#[derive(Debug)]
pub enum SlotCountTrackerError {
    CurrentNumFreeLessThanStoredNumFree,
    ReportOccupiedCountGreaterThanStoredNumFree,
}
