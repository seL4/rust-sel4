#![no_std]
#![feature(sync_unsafe_cell)]

use core::cell::SyncUnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

pub struct ImmediateSyncOnceCell<T> {
    init_started: AtomicBool,
    init_completed: AtomicBool,
    inner: SyncUnsafeCell<Option<T>>,
}

impl<T> ImmediateSyncOnceCell<T> {
    pub const fn new() -> Self {
        Self {
            init_started: AtomicBool::new(false),
            init_completed: AtomicBool::new(false),
            inner: SyncUnsafeCell::new(None),
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
            let slot = unsafe { &mut *self.inner.get() };
            *slot = Some(value);
            self.init_completed.store(true, Ordering::SeqCst);
            Ok(())
        }
    }
}
