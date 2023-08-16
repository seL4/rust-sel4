#![no_std]
#![feature(sync_unsafe_cell)]

use core::cell::SyncUnsafeCell;

#[repr(transparent)]
pub struct ImmutableCell<T: ?Sized> {
    value: SyncUnsafeCell<T>,
}

impl<T: Default> Default for ImmutableCell<T> {
    fn default() -> ImmutableCell<T> {
        ImmutableCell::new(Default::default())
    }
}

impl<T> From<T> for ImmutableCell<T> {
    fn from(t: T) -> ImmutableCell<T> {
        ImmutableCell::new(t)
    }
}

impl<T> ImmutableCell<T> {
    pub const fn new(value: T) -> Self {
        Self {
            value: SyncUnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> ImmutableCell<T> {
    pub fn get(&self) -> &T {
        unsafe { self.value.get().as_ref().unwrap() }
    }
}
