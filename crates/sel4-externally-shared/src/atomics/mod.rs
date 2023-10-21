//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;
use core::marker::PhantomData;
use core::ptr::NonNull;

use volatile::access::{ReadOnly, ReadWrite, WriteOnly};

mod generic;
mod ops;
mod ordering;

use ordering::OrderingExhaustive;

#[repr(transparent)]
pub struct AtomicPtr<'a, T, A = ReadWrite> {
    pointer: NonNull<T>,
    reference: PhantomData<&'a T>,
    access: PhantomData<A>,
}

impl<'a, T, A> Copy for AtomicPtr<'a, T, A> {}

impl<T, A> Clone for AtomicPtr<'_, T, A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, A> fmt::Debug for AtomicPtr<'_, T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AtomicPtr")
            .field("pointer", &self.pointer)
            .field("access", &self.access)
            .finish()
    }
}

impl<'a, T, A> AtomicPtr<'a, T, A> {
    pub const unsafe fn new(pointer: NonNull<T>) -> Self {
        AtomicPtr {
            pointer,
            reference: PhantomData,
            access: PhantomData,
        }
    }

    pub fn as_raw_ptr(self) -> NonNull<T> {
        self.pointer
    }

    pub fn read_only(self) -> AtomicPtr<'a, T, ReadOnly> {
        unsafe { AtomicPtr::new(self.pointer) }
    }

    pub fn write_only(self) -> AtomicPtr<'a, T, WriteOnly> {
        unsafe { AtomicPtr::new(self.pointer) }
    }
}

pub unsafe trait Atomic: AtomicSealed + Copy {
    const IS_SIGNED: bool;
}

use sealing::AtomicSealed;

mod sealing {
    pub trait AtomicSealed {}
}

macro_rules! impl_atomic {
    ($t:ty, $target_has_atomic_key:literal, $is_signed:literal) => {
        // TODO these attributes are overly conservative
        #[cfg(target_has_atomic = $target_has_atomic_key)]
        #[cfg(target_has_atomic_equal_alignment = $target_has_atomic_key)]
        unsafe impl Atomic for $t {
            const IS_SIGNED: bool = $is_signed;
        }

        impl AtomicSealed for $t {}
    };
}

macro_rules! impl_atomic_for_each_signedness {
    ($t_unsigned:ty, $t_signed:ty, $target_has_atomic_key:literal) => {
        impl_atomic!($t_unsigned, $target_has_atomic_key, false);
        impl_atomic!($t_signed, $target_has_atomic_key, true);
    };
}

impl_atomic_for_each_signedness!(u8, i8, "8");
impl_atomic_for_each_signedness!(u16, i16, "16");
impl_atomic_for_each_signedness!(u32, i32, "32");
impl_atomic_for_each_signedness!(u64, i64, "64");
impl_atomic_for_each_signedness!(u128, i128, "128");

#[cfg(target_pointer_width = "32")]
impl_atomic_for_each_signedness!(usize, isize, "32");

#[cfg(target_pointer_width = "64")]
impl_atomic_for_each_signedness!(usize, isize, "64");
