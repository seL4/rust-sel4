//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

use core::{
    ops::{self, Bound, Range, RangeBounds, RangeTo},
    ptr::{self, NonNull},
    slice::SliceIndex,
};

pub(crate) fn non_null_slice_len<T>(p: NonNull<[T]>) -> usize {
    mut_ptr_slice_len(p.as_ptr())
}

pub(crate) fn mut_ptr_slice_len<T>(p: *mut [T]) -> usize {
    ptr_meta::metadata(p)
}

pub(crate) fn non_null_slice_as_mut_ptr<T>(p: NonNull<[T]>) -> *mut T {
    mut_ptr_slice_as_mut_ptr(p.as_ptr())
}

pub(crate) fn mut_ptr_slice_as_mut_ptr<T>(p: *mut [T]) -> *mut T {
    p as *mut T
}

pub(crate) fn non_null_index<I, T>(
    p: NonNull<[T]>,
    index: I,
) -> NonNull<<I as SliceIndex<[T]>>::Output>
where
    I: AbstractPtrSliceIndex<[T]> + SliceIndex<[()]> + Clone,
{
    NonNull::new(index.abstract_ptr_slice_index(p.as_ptr())).unwrap()
}

pub(crate) fn range<R>(range: R, bounds: RangeTo<usize>) -> Range<usize>
where
    R: RangeBounds<usize>,
{
    let len = bounds.end;

    let start = match range.start_bound() {
        Bound::Included(&start) => start,
        Bound::Excluded(start) => start
            .checked_add(1)
            .unwrap_or_else(|| panic!("attempted to index slice from after maximum usize")),
        Bound::Unbounded => 0,
    };

    let end = match range.end_bound() {
        Bound::Included(end) => end
            .checked_add(1)
            .unwrap_or_else(|| panic!("attempted to index slice up to maximum usize")),
        Bound::Excluded(&end) => end,
        Bound::Unbounded => len,
    };

    if start > end {
        panic!("slice index starts at {start} but ends at {end}",)
    }
    if end > len {
        panic!("range end index {end} out of range for slice of length {len}",)
    }

    Range { start, end }
}

#[allow(private_bounds)]
pub trait AbstractPtrSliceIndex<T: ?Sized>:
    SliceIndex<T> + AbstractPtrSliceIndexInternal<T>
{
}

pub(crate) trait AbstractPtrSliceIndexInternal<T: ?Sized>: SliceIndex<T> {
    fn abstract_ptr_slice_index(self, slice: *mut T) -> *mut Self::Output;
}

impl<T> AbstractPtrSliceIndex<[T]> for usize {}

impl<T> AbstractPtrSliceIndexInternal<[T]> for usize {
    fn abstract_ptr_slice_index(self, slice: *mut [T]) -> *mut Self::Output {
        bounds_check(mut_ptr_slice_len(slice), self);
        mut_ptr_slice_as_mut_ptr(slice).wrapping_offset(self.try_into().unwrap())
    }
}

impl<T> AbstractPtrSliceIndex<[T]> for Range<usize> {}

impl<T> AbstractPtrSliceIndexInternal<[T]> for Range<usize> {
    fn abstract_ptr_slice_index(self, slice: *mut [T]) -> *mut Self::Output {
        bounds_check(mut_ptr_slice_len(slice), self.clone());
        ptr::slice_from_raw_parts_mut(self.start.abstract_ptr_slice_index(slice), self.len())
    }
}

macro_rules! slice_index_impl {
    ($t:ty) => {
        impl<T> AbstractPtrSliceIndex<[T]> for $t {}

        impl<T> AbstractPtrSliceIndexInternal<[T]> for $t {
            fn abstract_ptr_slice_index(self, slice: *mut [T]) -> *mut Self::Output {
                range(self, ..mut_ptr_slice_len(slice)).abstract_ptr_slice_index(slice)
            }
        }
    };
}

slice_index_impl!((Bound<usize>, Bound<usize>));
slice_index_impl!(ops::RangeFrom<usize>);
slice_index_impl!(ops::RangeFull);
slice_index_impl!(ops::RangeInclusive<usize>);
slice_index_impl!(ops::RangeTo<usize>);
slice_index_impl!(ops::RangeToInclusive<usize>);

fn bounds_check(len: usize, index: impl SliceIndex<[()]>) {
    const MAX_ARRAY: [(); usize::MAX] = [(); usize::MAX];

    let bound_check_slice = &MAX_ARRAY[..len];
    let _ = &bound_check_slice[index];
}
