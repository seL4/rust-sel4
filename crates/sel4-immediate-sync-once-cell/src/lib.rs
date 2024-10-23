//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

pub struct ImmediateSyncOnceCell<T> {
    init_started: AtomicBool,
    init_completed: AtomicBool,
    inner: UnsafeCell<Option<T>>,
}

unsafe impl<T> Sync for ImmediateSyncOnceCell<T> {}

impl<T> Default for ImmediateSyncOnceCell<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ImmediateSyncOnceCell<T> {
    pub const fn new() -> Self {
        Self {
            init_started: AtomicBool::new(false),
            init_completed: AtomicBool::new(false),
            inner: UnsafeCell::new(None),
        }
    }

    pub fn get(&self) -> Option<&T> {
        if self.init_completed.load(Ordering::Acquire) {
            Some(unsafe { self.inner.get().as_ref().unwrap().as_ref().unwrap() })
        } else {
            None
        }
    }

    pub fn set(&self, value: T) -> Result<(), T> {
        if self.init_started.swap(true, Ordering::SeqCst) {
            Err(value)
        } else {
            unsafe {
                *self.inner.get().as_mut().unwrap() = Some(value);
            }
            self.init_completed.store(true, Ordering::SeqCst);
            Ok(())
        }
    }
}
