//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

use core::sync::atomic::Ordering;

use crate::{AbstractPtr, memory_type::AtomicOps};

impl<M, T, A> AbstractPtr<'_, M, T, A>
where
    M: AtomicOps<T>,
{
    #[inline]
    pub fn atomic_load(self, order: Ordering) -> M::Value {
        unsafe { M::atomic_load(self.pointer.as_ptr().cast_const(), order) }
    }

    #[inline]
    pub fn atomic_store(self, val: M::Value, order: Ordering) {
        unsafe {
            M::atomic_store(self.pointer.as_ptr(), val, order);
        }
    }

    #[inline]
    pub fn atomic_swap(self, val: M::Value, order: Ordering) -> M::Value {
        unsafe { M::atomic_swap(self.pointer.as_ptr(), val, order) }
    }

    #[inline]
    pub fn atomic_compare_exchange(
        self,
        current: M::Value,
        new: M::Value,
        success: Ordering,
        failure: Ordering,
    ) -> Result<M::Value, M::Value> {
        unsafe { M::atomic_compare_exchange(self.pointer.as_ptr(), current, new, success, failure) }
    }

    #[inline]
    pub fn atomic_compare_exchange_weak(
        self,
        current: M::Value,
        new: M::Value,
        success: Ordering,
        failure: Ordering,
    ) -> Result<M::Value, M::Value> {
        unsafe {
            M::atomic_compare_exchange_weak(self.pointer.as_ptr(), current, new, success, failure)
        }
    }

    #[inline]
    pub fn atomic_fetch_add(self, val: M::Value, order: Ordering) -> M::Value {
        unsafe { M::atomic_add(self.pointer.as_ptr(), val, order) }
    }

    #[inline]
    pub fn atomic_fetch_sub(self, val: M::Value, order: Ordering) -> M::Value {
        unsafe { M::atomic_sub(self.pointer.as_ptr(), val, order) }
    }

    #[inline]
    pub fn atomic_fetch_and(self, val: M::Value, order: Ordering) -> M::Value {
        unsafe { M::atomic_and(self.pointer.as_ptr(), val, order) }
    }

    #[inline]
    pub fn atomic_fetch_nand(self, val: M::Value, order: Ordering) -> M::Value {
        unsafe { M::atomic_nand(self.pointer.as_ptr(), val, order) }
    }

    #[inline]
    pub fn atomic_fetch_or(self, val: M::Value, order: Ordering) -> M::Value {
        unsafe { M::atomic_or(self.pointer.as_ptr(), val, order) }
    }

    #[inline]
    pub fn atomic_fetch_xor(self, val: M::Value, order: Ordering) -> M::Value {
        unsafe { M::atomic_xor(self.pointer.as_ptr(), val, order) }
    }

    #[inline]
    pub fn atomic_fetch_update<F>(
        self,
        set_order: Ordering,
        fetch_order: Ordering,
        mut f: F,
    ) -> Result<M::Value, M::Value>
    where
        M::Value: Copy,
        F: FnMut(M::Value) -> Option<M::Value>,
    {
        let mut prev = self.atomic_load(fetch_order);
        while let Some(next) = f(prev) {
            match self.atomic_compare_exchange_weak(prev, next, set_order, fetch_order) {
                x @ Ok(_) => return x,
                Err(next_prev) => prev = next_prev,
            }
        }
        Err(prev)
    }

    #[inline]
    pub fn atomic_fetch_max(self, val: M::Value, order: Ordering) -> M::Value {
        unsafe { M::atomic_max(self.pointer.as_ptr(), val, order) }
    }

    #[inline]
    pub fn atomic_fetch_min(self, val: M::Value, order: Ordering) -> M::Value {
        unsafe { M::atomic_min(self.pointer.as_ptr(), val, order) }
    }
}
