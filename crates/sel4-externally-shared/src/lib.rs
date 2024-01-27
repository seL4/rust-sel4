//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![cfg_attr(feature = "unstable", feature(core_intrinsics))]
#![cfg_attr(feature = "unstable", allow(internal_features))]

use core::ptr::NonNull;

use volatile::access::{Access, ReadWrite};

pub use volatile::{access, map_field, VolatilePtr, VolatileRef};

pub mod ops;

pub use sel4_atomic_ptr::{Atomic, AtomicPtr};

// TODO
pub type ExternallySharedOps = ops::ZerocopyOps<ops::NormalOps>;
// pub type ExternallySharedOps = ops::ZerocopyOps<ops::VolatileOps>;
// pub type ExternallySharedOps = ops::ZerocopyOps<ops::BytewiseOps<ops::UnorderedAtomicOps>>;

pub type ExternallySharedRef<'a, T, A = ReadWrite> = VolatileRef<'a, T, A, ExternallySharedOps>;
pub type ExternallySharedPtr<'a, T, A = ReadWrite> = VolatilePtr<'a, T, A, ExternallySharedOps>;

pub trait ExternallySharedRefExt<'a, T: ?Sized, A: Access> {
    #[allow(clippy::missing_safety_doc)]
    #[allow(clippy::new_ret_no_self)]
    unsafe fn new(pointer: NonNull<T>) -> ExternallySharedRef<'a, T, A>;
}

impl<'a, T: ?Sized, A: Access> ExternallySharedRefExt<'a, T, A> for ExternallySharedRef<'a, T, A> {
    unsafe fn new(pointer: NonNull<T>) -> ExternallySharedRef<'a, T, A> {
        unsafe {
            VolatileRef::new_restricted_with_ops(Default::default(), Default::default(), pointer)
        }
    }
}

pub trait ExternallySharedPtrExt<'a, T: ?Sized> {
    fn atomic(self) -> AtomicPtr<'a, T>
    where
        T: Atomic;
}

impl<'a, T: ?Sized> ExternallySharedPtrExt<'a, T> for ExternallySharedPtr<'a, T, ReadWrite> {
    fn atomic(self) -> AtomicPtr<'a, T>
    where
        T: Atomic,
    {
        let p = self.as_raw_ptr();
        assert_eq!(p.as_ptr().align_offset(T::ALIGNMENT), 0);
        unsafe { AtomicPtr::new(p) }
    }
}
