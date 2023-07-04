#![no_std]
#![feature(allocator_api)]
#![feature(btree_cursors)]
#![feature(btreemap_alloc)]
#![feature(int_roundings)]
#![feature(pointer_is_aligned)]
#![feature(slice_ptr_get)]
#![feature(strict_provenance)]

extern crate alloc;

use core::alloc::Layout;
use core::ptr::NonNull;

type Offset = usize;
type Size = usize;
type Align = usize;

mod basic;
mod bump;

pub use basic::Basic;
pub use bump::Bump;

const MIN_ALLOCATION_SIZE: Size = 1;

pub trait AbstractBounceBufferAllocator {
    type Error;

    fn allocate(&mut self, layout: Layout) -> Result<Offset, Self::Error>;

    fn deallocate(&mut self, offset: Offset, size: Size);
}

pub struct BounceBufferAllocator<T> {
    abstract_allocator: T,
    region: NonNull<[u8]>,
    max_alignment: Align,
}

unsafe impl<T: Send> Send for BounceBufferAllocator<T> {}

impl<T> BounceBufferAllocator<T> {
    pub unsafe fn new(abstract_allocator: T, region: NonNull<[u8]>, max_alignment: Align) -> Self {
        assert!(max_alignment.is_power_of_two());
        assert!(region.as_ptr().is_aligned_to(max_alignment));
        Self {
            abstract_allocator,
            region,
            max_alignment,
        }
    }

    pub fn ptr_to_offset(&self, ptr: NonNull<u8>) -> Offset {
        assert!(ptr >= self.region.as_non_null_ptr());
        let offset = ptr.addr().get() - self.region.as_non_null_ptr().addr().get();
        assert!(offset <= self.region.len());
        offset
    }
}

impl<T: AbstractBounceBufferAllocator> BounceBufferAllocator<T> {
    pub fn allocate(&mut self, layout: Layout) -> Result<NonNull<[u8]>, T::Error> {
        assert!(layout.align() <= self.max_alignment);
        assert!(layout.size() >= MIN_ALLOCATION_SIZE);
        let offset = self.abstract_allocator.allocate(layout)?;
        assert!(offset + layout.size() <= self.region.len());
        let ptr = self
            .region
            .as_non_null_ptr()
            .map_addr(|addr| addr.checked_add(offset).unwrap());
        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
    }

    pub fn allocate_zeroed(&mut self, layout: Layout) -> Result<NonNull<[u8]>, T::Error> {
        let ptr = self.allocate(layout)?;
        unsafe {
            ptr.as_non_null_ptr().as_ptr().write_bytes(0, ptr.len());
        }
        Ok(ptr)
    }

    pub unsafe fn deallocate(&mut self, ptr: NonNull<[u8]>) {
        let offset = self.ptr_to_offset(ptr.as_non_null_ptr());
        self.abstract_allocator.deallocate(offset, ptr.len())
    }
}
