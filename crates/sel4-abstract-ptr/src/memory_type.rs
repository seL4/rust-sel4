//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

use core::sync::atomic::Ordering;

pub trait MemoryType {}

pub trait UnitaryOps<T>: MemoryType {
    unsafe fn read(src: *const T) -> T;

    unsafe fn write(dst: *mut T, src: T);
}

pub trait BulkOps<T>: MemoryType {
    unsafe fn memmove(dst: *mut T, src: *const T, count: usize);

    unsafe fn memcpy_into(dst: *mut T, src: *const T, count: usize);

    unsafe fn memcpy_from(dst: *mut T, src: *const T, count: usize);

    unsafe fn memset(dst: *mut T, val: u8, count: usize);
}

pub trait AtomicOps<T>: MemoryType {
    unsafe fn atomic_store(dst: *mut T, val: T, order: Ordering);

    unsafe fn atomic_load(dst: *const T, order: Ordering) -> T;

    unsafe fn atomic_swap(dst: *mut T, val: T, order: Ordering) -> T;

    unsafe fn atomic_add(dst: *mut T, val: T, order: Ordering) -> T;

    unsafe fn atomic_sub(dst: *mut T, val: T, order: Ordering) -> T;

    unsafe fn atomic_compare_exchange(
        dst: *mut T,
        old: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T>;

    unsafe fn atomic_compare_exchange_weak(
        dst: *mut T,
        old: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T>;

    unsafe fn atomic_and(dst: *mut T, val: T, order: Ordering) -> T;

    unsafe fn atomic_nand(dst: *mut T, val: T, order: Ordering) -> T;

    unsafe fn atomic_or(dst: *mut T, val: T, order: Ordering) -> T;

    unsafe fn atomic_xor(dst: *mut T, val: T, order: Ordering) -> T;

    unsafe fn atomic_max(dst: *mut T, val: T, order: Ordering) -> T;

    unsafe fn atomic_min(dst: *mut T, val: T, order: Ordering) -> T;
}
