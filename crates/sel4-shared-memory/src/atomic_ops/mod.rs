//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

use core::mem;
use core::sync::atomic::{self, Ordering};

use zerocopy::{FromBytes, IntoBytes};

use sel4_abstract_ptr::memory_type::AtomicOps;

use crate::SharedMemory;

mod generic;
mod ordering;

#[allow(private_bounds)]
pub trait HasAtomics: Copy + FromBytes + IntoBytes + HasAtomicsSealed {}

trait HasAtomicsSealed {
    const ALIGNMENT: usize;
    const IS_SIGNED: bool;
}

macro_rules! impl_atomic {
    (
        $t:ty,
        $target_has_atomic_key:literal,
        $is_signed:literal,
        $analog_for_alignment:path
    ) => {
        #[cfg(target_has_atomic = $target_has_atomic_key)]
        impl HasAtomics for $t {}

        #[cfg(target_has_atomic = $target_has_atomic_key)]
        impl HasAtomicsSealed for $t {
            const ALIGNMENT: usize = mem::align_of::<$analog_for_alignment>();
            const IS_SIGNED: bool = $is_signed;
        }
    };
}

macro_rules! impl_atomic_for_each_signedness {
    (
        $t_unsigned:ty,
        $t_signed:ty,
        $target_has_atomic_key:literal,
        $unsigned_analog_for_alignment:path,
        $signed_analog_for_alignment:path
    ) => {
        impl_atomic!(
            $t_unsigned,
            $target_has_atomic_key,
            false,
            $unsigned_analog_for_alignment
        );
        impl_atomic!(
            $t_signed,
            $target_has_atomic_key,
            true,
            $signed_analog_for_alignment
        );
    };
}

impl_atomic_for_each_signedness!(u8, i8, "8", atomic::AtomicU8, atomic::AtomicI8);
impl_atomic_for_each_signedness!(u16, i16, "16", atomic::AtomicU16, atomic::AtomicI16);
impl_atomic_for_each_signedness!(u32, i32, "32", atomic::AtomicU32, atomic::AtomicI32);
impl_atomic_for_each_signedness!(u64, i64, "64", atomic::AtomicU64, atomic::AtomicI64);

#[cfg(target_pointer_width = "32")]
impl_atomic_for_each_signedness!(usize, isize, "32", atomic::AtomicUsize, atomic::AtomicIsize);

#[cfg(target_pointer_width = "64")]
impl_atomic_for_each_signedness!(usize, isize, "64", atomic::AtomicUsize, atomic::AtomicIsize);

impl<T: HasAtomics> AtomicOps<T> for SharedMemory {
    #[inline]
    unsafe fn atomic_store(dst: *mut T, val: T, order: Ordering) {
        unsafe { generic::atomic_store(dst, val, order.into()) }
    }

    #[inline]
    unsafe fn atomic_load(dst: *const T, order: Ordering) -> T {
        unsafe { generic::atomic_load(dst, order.into()) }
    }

    #[inline]
    unsafe fn atomic_swap(dst: *mut T, val: T, order: Ordering) -> T {
        unsafe { generic::atomic_swap(dst, val, order.into()) }
    }

    #[inline]
    unsafe fn atomic_add(dst: *mut T, val: T, order: Ordering) -> T {
        unsafe { generic::atomic_add(dst, val, order.into()) }
    }

    #[inline]
    unsafe fn atomic_sub(dst: *mut T, val: T, order: Ordering) -> T {
        unsafe { generic::atomic_sub(dst, val, order.into()) }
    }

    #[inline]
    unsafe fn atomic_compare_exchange(
        dst: *mut T,
        old: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T> {
        unsafe { generic::atomic_compare_exchange(dst, old, new, success.into(), failure.into()) }
    }

    #[inline]
    unsafe fn atomic_compare_exchange_weak(
        dst: *mut T,
        old: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T> {
        unsafe {
            generic::atomic_compare_exchange_weak(dst, old, new, success.into(), failure.into())
        }
    }

    #[inline]
    unsafe fn atomic_and(dst: *mut T, val: T, order: Ordering) -> T {
        unsafe { generic::atomic_and(dst, val, order.into()) }
    }

    #[inline]
    unsafe fn atomic_nand(dst: *mut T, val: T, order: Ordering) -> T {
        unsafe { generic::atomic_nand(dst, val, order.into()) }
    }

    #[inline]
    unsafe fn atomic_or(dst: *mut T, val: T, order: Ordering) -> T {
        unsafe { generic::atomic_or(dst, val, order.into()) }
    }

    #[inline]
    unsafe fn atomic_xor(dst: *mut T, val: T, order: Ordering) -> T {
        unsafe { generic::atomic_xor(dst, val, order.into()) }
    }

    #[inline]
    unsafe fn atomic_max(dst: *mut T, val: T, order: Ordering) -> T {
        unsafe {
            if T::IS_SIGNED {
                generic::atomic_max(dst, val, order.into())
            } else {
                generic::atomic_umax(dst, val, order.into())
            }
        }
    }

    #[inline]
    unsafe fn atomic_min(dst: *mut T, val: T, order: Ordering) -> T {
        unsafe {
            if T::IS_SIGNED {
                generic::atomic_min(dst, val, order.into())
            } else {
                generic::atomic_umin(dst, val, order.into())
            }
        }
    }
}
