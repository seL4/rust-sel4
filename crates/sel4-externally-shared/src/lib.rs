#![no_std]
#![feature(cfg_target_has_atomic_equal_alignment)]
#![feature(concat_idents)]
#![feature(core_intrinsics)]

use core::ptr::NonNull;

use volatile::access::{Access, ReadWrite};

pub use volatile::{access, map_field, VolatilePtr, VolatileRef};

mod atomics;

pub mod ops;

pub use atomics::{Atomic, AtomicPtr};

// TODO
pub type ExternallySharedOps = ops::ZerocopyOps<ops::NormalOps>;
// pub type ExternallySharedOps = ops::ZerocopyOps<ops::VolatileOps>;
// pub type ExternallySharedOps = ops::ZerocopyOps<ops::BytewiseOps<ops::UnorderedAtomicOps>>;

pub type ExternallySharedRef<'a, T, A = ReadWrite> = VolatileRef<'a, T, A, ExternallySharedOps>;

pub type ExternallySharedPtr<'a, T, A = ReadWrite> = VolatilePtr<'a, T, A, ExternallySharedOps>;

pub trait ExternallySharedRefExt<'a, T: ?Sized, A: Access> {
    unsafe fn new(pointer: NonNull<T>) -> ExternallySharedRef<'a, T, A>;
}

impl<'a, T: ?Sized, A: Access> ExternallySharedRefExt<'a, T, A> for ExternallySharedRef<'a, T, A> {
    unsafe fn new(pointer: NonNull<T>) -> ExternallySharedRef<'a, T, A> {
        unsafe {
            VolatileRef::new_restricted_with_ops(Default::default(), Default::default(), pointer)
        }
    }
}

pub trait ExternallySharedPtrExt<'a, T: ?Sized, A> {
    fn atomic(self) -> AtomicPtr<'a, T, A>
    where
        T: Atomic;
}

impl<'a, T: ?Sized, A> ExternallySharedPtrExt<'a, T, A> for ExternallySharedPtr<'a, T, A> {
    fn atomic(self) -> AtomicPtr<'a, T, A>
    where
        T: Atomic,
    {
        unsafe { AtomicPtr::new(self.as_raw_ptr()) }
    }
}
