//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

// NOTE(rustc_wishlist) use SyncUnsafeCell once #![feature(sync_unsafe_cell)] stabilizes
pub struct OneRefCell<T> {
    taken: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for OneRefCell<T> {}

impl<T: Default> Default for OneRefCell<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> From<T> for OneRefCell<T> {
    fn from(t: T) -> Self {
        Self::new(t)
    }
}

impl<T> OneRefCell<T> {
    pub const fn new(value: T) -> Self {
        Self {
            taken: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    pub fn take(&self) -> Result<&mut T, Error> {
        if self.taken.swap(true, Ordering::SeqCst) {
            Err(Error::AlreadyTaken)
        } else {
            let ptr = self.value.get();
            Ok(unsafe { ptr.as_mut() }.unwrap())
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Error {
    AlreadyTaken,
}
