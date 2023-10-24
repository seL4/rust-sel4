//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

use core::sync::atomic::Ordering;

use volatile::access::{Readable, Writable};

use super::{generic, Atomic, AtomicPtr};

impl<'a, T, A> AtomicPtr<'a, T, A> {
    fn as_mut_ptr(self) -> *mut T {
        self.pointer.as_ptr()
    }

    fn as_const_ptr(self) -> *const T {
        self.as_mut_ptr().cast_const()
    }
}

impl<'a, T: Atomic, A: Readable> AtomicPtr<'a, T, A> {
    #[inline]
    pub fn load(self, order: Ordering) -> T {
        // SAFETY: data races are prevented by atomic intrinsics.
        unsafe { generic::atomic_load(self.as_const_ptr(), order.into()) }
    }
}

impl<'a, T: Atomic, A: Readable + Writable> AtomicPtr<'a, T, A> {
    #[inline]
    pub fn store(self, val: T, order: Ordering) {
        // SAFETY: data races are prevented by atomic intrinsics.
        unsafe {
            generic::atomic_store(self.as_mut_ptr(), val, order.into());
        }
    }

    #[inline]
    pub fn swap(self, val: T, order: Ordering) -> T {
        // SAFETY: data races are prevented by atomic intrinsics.
        unsafe { generic::atomic_swap(self.as_mut_ptr(), val, order.into()) }
    }

    #[inline]
    pub fn compare_exchange(
        self,
        current: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T> {
        // SAFETY: data races are prevented by atomic intrinsics.
        unsafe {
            generic::atomic_compare_exchange(
                self.as_mut_ptr(),
                current,
                new,
                success.into(),
                failure.into(),
            )
        }
    }

    #[inline]
    pub fn compare_exchange_weak(
        self,
        current: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T> {
        // SAFETY: data races are prevented by atomic intrinsics.
        unsafe {
            generic::atomic_compare_exchange_weak(
                self.as_mut_ptr(),
                current,
                new,
                success.into(),
                failure.into(),
            )
        }
    }

    #[inline]
    pub fn fetch_add(self, val: T, order: Ordering) -> T {
        // SAFETY: data races are prevented by atomic intrinsics.
        unsafe { generic::atomic_add(self.as_mut_ptr(), val, order.into()) }
    }

    #[inline]
    pub fn fetch_sub(self, val: T, order: Ordering) -> T {
        // SAFETY: data races are prevented by atomic intrinsics.
        unsafe { generic::atomic_sub(self.as_mut_ptr(), val, order.into()) }
    }

    #[inline]
    pub fn fetch_and(self, val: T, order: Ordering) -> T {
        // SAFETY: data races are prevented by atomic intrinsics.
        unsafe { generic::atomic_and(self.as_mut_ptr(), val, order.into()) }
    }

    #[inline]
    pub fn fetch_nand(self, val: T, order: Ordering) -> T {
        // SAFETY: data races are prevented by atomic intrinsics.
        unsafe { generic::atomic_nand(self.as_mut_ptr(), val, order.into()) }
    }

    #[inline]
    pub fn fetch_or(self, val: T, order: Ordering) -> T {
        // SAFETY: data races are prevented by atomic intrinsics.
        unsafe { generic::atomic_or(self.as_mut_ptr(), val, order.into()) }
    }

    #[inline]
    pub fn fetch_xor(self, val: T, order: Ordering) -> T {
        // SAFETY: data races are prevented by atomic intrinsics.
        unsafe { generic::atomic_xor(self.as_mut_ptr(), val, order.into()) }
    }

    #[inline]
    pub fn fetch_update<F>(
        self,
        set_order: Ordering,
        fetch_order: Ordering,
        mut f: F,
    ) -> Result<T, T>
    where
        F: FnMut(T) -> Option<T>,
    {
        let mut prev = self.load(fetch_order.into());
        while let Some(next) = f(prev) {
            match self.compare_exchange_weak(prev, next, set_order.into(), fetch_order.into()) {
                x @ Ok(_) => return x,
                Err(next_prev) => prev = next_prev,
            }
        }
        Err(prev)
    }

    #[inline]
    pub fn fetch_max(self, val: T, order: Ordering) -> T {
        // SAFETY: data races are prevented by atomic intrinsics.
        unsafe {
            if T::IS_SIGNED {
                generic::atomic_max(self.as_mut_ptr(), val, order.into())
            } else {
                generic::atomic_umax(self.as_mut_ptr(), val, order.into())
            }
        }
    }

    #[inline]
    pub fn fetch_min(self, val: T, order: Ordering) -> T {
        // SAFETY: data races are prevented by atomic intrinsics.
        unsafe {
            if T::IS_SIGNED {
                generic::atomic_min(self.as_mut_ptr(), val, order.into())
            } else {
                generic::atomic_umin(self.as_mut_ptr(), val, order.into())
            }
        }
    }
}
