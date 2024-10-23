//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::cell::UnsafeCell;

#[repr(transparent)]
pub struct ImmutableCell<T: ?Sized> {
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for ImmutableCell<T> {}

impl<T: Default> Default for ImmutableCell<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> From<T> for ImmutableCell<T> {
    fn from(t: T) -> Self {
        Self::new(t)
    }
}

impl<T> ImmutableCell<T> {
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> ImmutableCell<T> {
    pub fn get(&self) -> &T {
        unsafe { self.value.get().as_ref().unwrap() }
    }
}
