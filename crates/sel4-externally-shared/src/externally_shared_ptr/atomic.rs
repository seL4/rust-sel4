use core::sync::atomic;

use crate::{
    access::{Readable, Writable},
    ExternallySharedPtr,
};

pub trait AtomicPrimitive {
    type Atomic;

    unsafe fn wrap_atomic<'a>(ptr: *mut Self) -> &'a Self::Atomic;
}

macro_rules! atomic_primitive_impl {
    ($prim:path, $wrapper:path) => {
        impl AtomicPrimitive for $prim {
            type Atomic = $wrapper;

            unsafe fn wrap_atomic<'a>(ptr: *mut Self) -> &'a Self::Atomic {
                unsafe { Self::Atomic::from_ptr(ptr) }
            }
        }
    };
}

atomic_primitive_impl!(bool, atomic::AtomicBool);
atomic_primitive_impl!(i8, atomic::AtomicI8);
atomic_primitive_impl!(i16, atomic::AtomicI16);
atomic_primitive_impl!(i32, atomic::AtomicI32);
atomic_primitive_impl!(i64, atomic::AtomicI64);
atomic_primitive_impl!(isize, atomic::AtomicIsize);
atomic_primitive_impl!(u8, atomic::AtomicU8);
atomic_primitive_impl!(u16, atomic::AtomicU16);
atomic_primitive_impl!(u32, atomic::AtomicU32);
atomic_primitive_impl!(u64, atomic::AtomicU64);
atomic_primitive_impl!(usize, atomic::AtomicUsize);

impl<T> AtomicPrimitive for *mut T {
    type Atomic = atomic::AtomicPtr<T>;

    unsafe fn wrap_atomic<'a>(ptr: *mut Self) -> &'a Self::Atomic {
        unsafe { Self::Atomic::from_ptr(ptr) }
    }
}

impl<'a, T: AtomicPrimitive, A: Readable + Writable> ExternallySharedPtr<'a, T, A> {
    /// `TODO`
    pub fn with_atomic<R, F: FnOnce(&T::Atomic) -> R>(self, f: F) -> R {
        f(unsafe { T::wrap_atomic(self.as_raw_ptr().as_ptr()) })
    }
}
