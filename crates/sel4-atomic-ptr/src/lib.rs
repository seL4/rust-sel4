//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

#![no_std]
#![feature(core_intrinsics)]
#![allow(internal_features)]

use core::fmt;
use core::marker::PhantomData;
use core::mem;
use core::ptr::NonNull;
use core::sync::atomic;

mod generic;
mod ops;
mod ordering;

use ordering::OrderingExhaustive;

#[repr(transparent)]
pub struct AtomicPtr<'a, T> {
    pointer: NonNull<T>,
    reference: PhantomData<&'a T>,
}

impl<T> Copy for AtomicPtr<'_, T> {}

impl<T> Clone for AtomicPtr<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for AtomicPtr<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AtomicPtr").field(&self.pointer).finish()
    }
}

impl<T> AtomicPtr<'_, T> {
    /// # Safety
    ///
    /// Necessary but not sufficient:
    /// * `pointer` must be aligned to `align_of::<core::sync::atomic::Atomic*>()`
    #[allow(clippy::missing_safety_doc)]
    pub const unsafe fn new(pointer: NonNull<T>) -> Self {
        AtomicPtr {
            pointer,
            reference: PhantomData,
        }
    }

    pub fn as_raw_ptr(self) -> NonNull<T> {
        self.pointer
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe trait Atomic: AtomicSealed + Copy {}

use sealing::AtomicSealed;

mod sealing {
    pub trait AtomicSealed {
        const ALIGNMENT: usize;
        const IS_SIGNED: bool;
    }
}

macro_rules! impl_atomic {
    (
        $t:ty,
        $target_has_atomic_key:literal,
        $is_signed:literal,
        $analog_for_alignment:path
    ) => {
        // TODO these attributes are overly conservative
        #[cfg(target_has_atomic = $target_has_atomic_key)]
        unsafe impl Atomic for $t {}

        impl AtomicSealed for $t {
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
