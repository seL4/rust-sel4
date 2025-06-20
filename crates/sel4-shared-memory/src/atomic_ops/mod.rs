//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

use core::sync::atomic::{self, Ordering};

use aligned::Aligned;
use zerocopy::{FromBytes, IntoBytes};

use sel4_abstract_ptr::memory_type::AtomicOps;

use crate::SharedMemory;

mod generic;
mod ordering;

#[allow(private_bounds)]
pub trait Atomic: AtomicSealed {
    type Value: Copy + FromBytes + IntoBytes;
}

trait AtomicSealed {
    const IS_SIGNED: bool;
}

impl<A: Atomic> Atomic for Aligned<A, A::Value> {
    type Value = A::Value;
}

impl<A: Atomic> AtomicSealed for Aligned<A, A::Value> {
    const IS_SIGNED: bool = A::IS_SIGNED;
}

macro_rules! impl_atomic {
    (
        $atomic:path,
        $value:ty,
        $target_has_atomic_key:literal,
        $is_signed:literal
    ) => {
        #[cfg(target_has_atomic = $target_has_atomic_key)]
        impl Atomic for $atomic {
            type Value = $value;
        }

        #[cfg(target_has_atomic = $target_has_atomic_key)]
        impl AtomicSealed for $atomic {
            const IS_SIGNED: bool = $is_signed;
        }

        #[cfg(target_has_atomic = $target_has_atomic_key)]
        #[cfg(target_has_atomic_equal_alignment = $target_has_atomic_key)]
        impl Atomic for $value {
            type Value = $value;
        }

        #[cfg(target_has_atomic = $target_has_atomic_key)]
        #[cfg(target_has_atomic_equal_alignment = $target_has_atomic_key)]
        impl AtomicSealed for $value {
            const IS_SIGNED: bool = $is_signed;
        }
    };
}

macro_rules! impl_atomic_for_each_signedness {
    (
        $value_unsigned:ty,
        $value_signed:ty,
        $target_has_atomic_key:literal,
        $atomic_unsigned:path,
        $atomic_signed:path
    ) => {
        impl_atomic!(
            $atomic_unsigned,
            $value_unsigned,
            $target_has_atomic_key,
            false
        );
        impl_atomic!($atomic_signed, $value_signed, $target_has_atomic_key, true);
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

impl<T: Atomic> AtomicOps<T> for SharedMemory {
    type Value = T::Value;

    #[inline]
    unsafe fn atomic_store(dst: *mut T, val: Self::Value, order: Ordering) {
        unsafe { generic::atomic_store(dst.cast(), val, order.into()) }
    }

    #[inline]
    unsafe fn atomic_load(dst: *const T, order: Ordering) -> Self::Value {
        unsafe { generic::atomic_load(dst.cast(), order.into()) }
    }

    #[inline]
    unsafe fn atomic_swap(dst: *mut T, val: Self::Value, order: Ordering) -> Self::Value {
        unsafe { generic::atomic_swap(dst.cast(), val, order.into()) }
    }

    #[inline]
    unsafe fn atomic_add(dst: *mut T, val: Self::Value, order: Ordering) -> Self::Value {
        unsafe { generic::atomic_add(dst.cast(), val, order.into()) }
    }

    #[inline]
    unsafe fn atomic_sub(dst: *mut T, val: Self::Value, order: Ordering) -> Self::Value {
        unsafe { generic::atomic_sub(dst.cast(), val, order.into()) }
    }

    #[inline]
    unsafe fn atomic_compare_exchange(
        dst: *mut T,
        old: Self::Value,
        new: Self::Value,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self::Value, Self::Value> {
        unsafe {
            generic::atomic_compare_exchange(dst.cast(), old, new, success.into(), failure.into())
        }
    }

    #[inline]
    unsafe fn atomic_compare_exchange_weak(
        dst: *mut T,
        old: Self::Value,
        new: Self::Value,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self::Value, Self::Value> {
        unsafe {
            generic::atomic_compare_exchange_weak(
                dst.cast(),
                old,
                new,
                success.into(),
                failure.into(),
            )
        }
    }

    #[inline]
    unsafe fn atomic_and(dst: *mut T, val: Self::Value, order: Ordering) -> Self::Value {
        unsafe { generic::atomic_and(dst.cast(), val, order.into()) }
    }

    #[inline]
    unsafe fn atomic_nand(dst: *mut T, val: Self::Value, order: Ordering) -> Self::Value {
        unsafe { generic::atomic_nand(dst.cast(), val, order.into()) }
    }

    #[inline]
    unsafe fn atomic_or(dst: *mut T, val: Self::Value, order: Ordering) -> Self::Value {
        unsafe { generic::atomic_or(dst.cast(), val, order.into()) }
    }

    #[inline]
    unsafe fn atomic_xor(dst: *mut T, val: Self::Value, order: Ordering) -> Self::Value {
        unsafe { generic::atomic_xor(dst.cast(), val, order.into()) }
    }

    #[inline]
    unsafe fn atomic_max(dst: *mut T, val: Self::Value, order: Ordering) -> Self::Value {
        unsafe {
            if T::IS_SIGNED {
                generic::atomic_max(dst.cast(), val, order.into())
            } else {
                generic::atomic_umax(dst.cast(), val, order.into())
            }
        }
    }

    #[inline]
    unsafe fn atomic_min(dst: *mut T, val: Self::Value, order: Ordering) -> Self::Value {
        unsafe {
            if T::IS_SIGNED {
                generic::atomic_min(dst.cast(), val, order.into())
            } else {
                generic::atomic_umin(dst.cast(), val, order.into())
            }
        }
    }
}
