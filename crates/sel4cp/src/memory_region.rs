use core::marker::PhantomData;
use core::mem;
use core::ptr;
use core::slice;

use zerocopy::{AsBytes, FromBytes};

pub use sel4_shared::access::{ReadOnly, ReadWrite};
pub use sel4_shared::Volatile;

pub type MemoryRegion<T, A> = Volatile<<A as MemoryRegionAccess>::Ref<T>, A>;

pub unsafe fn new_memory_region<T: MemoryRegionTarget + ?Sized, A: MemoryRegionAccess>(
    start: A::Ptr<T::Element>,
    size_in_bytes: usize,
) -> MemoryRegion<T, A> {
    T::new_memory_region(start, size_in_bytes)
}

#[macro_export]
macro_rules! declare_memory_region {
    {
        <$target:ty, $access:ty>($symbol:ident, $size_in_bytes:expr)
    } => {
        {
            #[no_mangle]
            #[link_section = ".data"]
            static mut $symbol:
                <$access as $crate::memory_region::MemoryRegionAccess>::Ptr::<
                    <$target as $crate::memory_region::MemoryRegionTarget>::Element
                > =
                <
                    <$access as $crate::memory_region::MemoryRegionAccess>::Ptr::<
                        <$target as $crate::memory_region::MemoryRegionTarget>::Element
                    >
                    as $crate::memory_region::MemoryRegionPointer
                >::NULL;

            $crate::memory_region::new_memory_region::<$target, $access>(
                unsafe { $symbol },
                $size_in_bytes,
            )
        }
    }
}

pub use declare_memory_region;

// // //

pub trait MemoryRegionPointer: Copy {
    const NULL: Self;

    fn is_null(self) -> bool;

    fn is_aligned(self) -> bool;
}

impl<T> MemoryRegionPointer for *const T {
    const NULL: Self = ptr::null();

    fn is_null(self) -> bool {
        <*const T>::is_null(self)
    }

    fn is_aligned(self) -> bool {
        <*const T>::is_aligned(self)
    }
}

impl<T> MemoryRegionPointer for *mut T {
    const NULL: Self = ptr::null_mut();

    fn is_null(self) -> bool {
        <*mut T>::is_null(self)
    }

    fn is_aligned(self) -> bool {
        <*mut T>::is_aligned(self)
    }
}

pub trait MemoryRegionAccess: Sized {
    type Ptr<T>: MemoryRegionPointer;
    type Ref<T: 'static + ?Sized>;

    unsafe fn new_memory_region<T: ?Sized>(reference: Self::Ref<T>) -> MemoryRegion<T, Self>;

    unsafe fn ref_from_ptr<T>(pointer: Self::Ptr<T>) -> Option<Self::Ref<T>>;

    unsafe fn slice_from_raw_parts<T>(data: Self::Ptr<T>, len: usize) -> Self::Ref<[T]>;
}

impl MemoryRegionAccess for ReadOnly {
    type Ptr<T> = *const T;
    type Ref<T: 'static + ?Sized> = &'static T;

    unsafe fn new_memory_region<T: ?Sized>(reference: Self::Ref<T>) -> MemoryRegion<T, Self> {
        Volatile::new_read_only(reference)
    }

    unsafe fn ref_from_ptr<T>(pointer: Self::Ptr<T>) -> Option<Self::Ref<T>> {
        pointer.as_ref()
    }

    unsafe fn slice_from_raw_parts<T>(data: Self::Ptr<T>, len: usize) -> Self::Ref<[T]> {
        slice::from_raw_parts(data, len)
    }
}

impl MemoryRegionAccess for ReadWrite {
    type Ptr<T> = *mut T;
    type Ref<T: 'static + ?Sized> = &'static mut T;

    unsafe fn new_memory_region<T: ?Sized>(reference: Self::Ref<T>) -> MemoryRegion<T, Self> {
        Volatile::new(reference)
    }

    unsafe fn ref_from_ptr<T>(pointer: Self::Ptr<T>) -> Option<Self::Ref<T>> {
        pointer.as_mut()
    }

    unsafe fn slice_from_raw_parts<T>(data: Self::Ptr<T>, len: usize) -> Self::Ref<[T]> {
        slice::from_raw_parts_mut(data, len)
    }
}

pub trait MemoryRegionTarget: AsBytes + FromBytes {
    type Element;

    unsafe fn new_memory_region<A: MemoryRegionAccess>(
        start: A::Ptr<Self::Element>,
        size_in_bytes: usize,
    ) -> MemoryRegion<Self, A>;
}

impl<T: Sized + AsBytes + FromBytes> MemoryRegionTarget for T {
    type Element = T;

    unsafe fn new_memory_region<A: MemoryRegionAccess>(
        start: A::Ptr<Self::Element>,
        size_in_bytes: usize,
    ) -> MemoryRegion<Self, A> {
        assert!(!start.is_null());
        assert!(start.is_aligned());
        assert!(size_in_bytes >= mem::size_of::<Self::Element>());
        A::new_memory_region(unsafe { A::ref_from_ptr(start).unwrap() })
    }
}

impl<T: Sized + AsBytes + FromBytes> MemoryRegionTarget for [T] {
    type Element = T;

    unsafe fn new_memory_region<A: MemoryRegionAccess>(
        start: A::Ptr<Self::Element>,
        size_in_bytes: usize,
    ) -> MemoryRegion<Self, A> {
        assert!(!start.is_null());
        assert!(start.is_aligned());
        assert_eq!(size_in_bytes % mem::size_of::<Self::Element>(), 0);
        let len = size_in_bytes / mem::size_of::<Self::Element>();
        A::new_memory_region(unsafe { A::slice_from_raw_parts(start, len) })
    }
}

// // //

pub struct DeferredMemoryRegion<T: MemoryRegionTarget + ?Sized, A: MemoryRegionAccess> {
    size_in_bytes: usize,
    get_start: fn() -> A::Ptr<T::Element>,
    phantom: PhantomData<A>,
}

impl<T: MemoryRegionTarget + ?Sized, A: MemoryRegionAccess> DeferredMemoryRegion<T, A> {
    pub const fn new(size_in_bytes: usize, get_start: fn() -> A::Ptr<T::Element>) -> Self {
        // TODO check `size_in_bytes` at compile time using a const trait like `MemoryRegionTarget`
        Self {
            size_in_bytes,
            get_start,
            phantom: PhantomData,
        }
    }

    pub unsafe fn construct(&self) -> MemoryRegion<T, A> {
        new_memory_region((self.get_start)(), self.size_in_bytes)
    }
}

#[macro_export]
macro_rules! declare_deferred_memory_region {
    {
        <$target:ty, $access:ty>($symbol:ident, $size_in_bytes:expr)
    } => {
        $crate::memory_region::DeferredMemoryRegion::new($size_in_bytes, || {
            #[no_mangle]
            #[link_section = ".data"]
            static mut $symbol:
                <$access as $crate::memory_region::MemoryRegionAccess>::Ptr::<
                    <$target as $crate::memory_region::MemoryRegionTarget>::Element
                > =
                <
                    <$access as $crate::memory_region::MemoryRegionAccess>::Ptr::<
                        <$target as $crate::memory_region::MemoryRegionTarget>::Element
                    > as $crate::memory_region::MemoryRegionPointer
                >::NULL;

            unsafe { $symbol }
        })
    }
}

pub use declare_deferred_memory_region;

// // //

// HACK

#[cfg(feature = "alloc")]
pub use volatile_slice_ext::VolatileSliceExt;

#[cfg(feature = "alloc")]
mod volatile_slice_ext {
    use alloc::vec::Vec;
    use core::mem::MaybeUninit;
    use core::ops::Deref;

    use sel4_shared::Volatile;

    pub trait VolatileSliceExt<T, R, A>
    where
        R: Deref<Target = [T]>,
    {
        fn volatile_slice_ext_inner(&self) -> &Volatile<R, A>;

        fn len(&self) -> usize {
            // HACK HACK HACK
            // TODO upstream proper `len` method
            let mut len = None;
            self.volatile_slice_ext_inner().map(|x| {
                len = Some(x.len());
                x
            });
            len.unwrap()
        }

        fn copy_to_vec(&self) -> Vec<T>
        where
            T: Copy,
        {
            vec_from_write_only_init(self.len(), |buf| {
                self.volatile_slice_ext_inner().copy_into_slice(buf);
            })
        }
    }

    impl<T, R, A> VolatileSliceExt<T, R, A> for Volatile<R, A>
    where
        R: Deref<Target = [T]>,
    {
        fn volatile_slice_ext_inner(&self) -> &Volatile<R, A> {
            self
        }
    }

    fn vec_from_write_only_init<T>(n: usize, f: impl FnOnce(&mut [T])) -> Vec<T> {
        let mut v = Vec::with_capacity(n);
        let uninit = v.spare_capacity_mut();
        unsafe {
            f(MaybeUninit::slice_assume_init_mut(uninit));
            v.set_len(n);
        }
        v
    }
}
