use core::mem;
use core::ptr;
use core::slice;

use zerocopy::{AsBytes, FromBytes};

pub use sel4_externally_shared::access::{ReadOnly, ReadWrite};
pub use sel4_externally_shared::ExternallyShared;

pub type MemoryRegion<T, A> = ExternallyShared<<A as MemoryRegionAccess>::Ref<T>, A>;

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
        ExternallyShared::new_read_only(reference)
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
        ExternallyShared::new(reference)
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
