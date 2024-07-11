//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::cell::Cell;
use core::sync::atomic::{AtomicBool, Ordering};

pub struct PanickingRawMutex {
    locked: AtomicBool,
}

impl PanickingRawMutex {
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }
}

impl Default for PanickingRawMutex {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl lock_api::RawMutex for PanickingRawMutex {
    type GuardMarker = lock_api::GuardNoSend; // TODO

    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = Self::new();

    fn lock(&self) {
        if !self.try_lock() {
            panic!("lock contention")
        }
    }

    fn try_lock(&self) -> bool {
        let was_locked = self.locked.swap(true, Ordering::Acquire);
        !was_locked
    }

    unsafe fn unlock(&self) {
        self.locked.store(false, Ordering::Release)
    }
}

pub struct UnsyncPanickingRawMutex {
    locked: Cell<bool>,
}

impl UnsyncPanickingRawMutex {
    pub const fn new() -> Self {
        Self {
            locked: Cell::new(false),
        }
    }
}

impl Default for UnsyncPanickingRawMutex {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl lock_api::RawMutex for UnsyncPanickingRawMutex {
    type GuardMarker = lock_api::GuardNoSend; // TODO

    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = Self::new();

    fn lock(&self) {
        if !self.try_lock() {
            panic!("lock contention")
        }
    }

    fn try_lock(&self) -> bool {
        let was_locked = self.locked.replace(true);
        !was_locked
    }

    unsafe fn unlock(&self) {
        self.locked.set(false)
    }
}
