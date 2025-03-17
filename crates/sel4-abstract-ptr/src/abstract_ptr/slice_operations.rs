//
// Copyright 2024, Colias Group, LLC
// Copyright (c) 2020 Philipp Oppermann
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

use core::{
    ops::{Range, RangeBounds},
    ptr::{self, NonNull},
    slice::SliceIndex,
};

use crate::{
    access::{Access, Readable, Writable},
    core_ext::{
        non_null_index, non_null_slice_as_mut_ptr, non_null_slice_len, range, AbstractPtrSliceIndex,
    },
    memory_type::BulkOps,
    AbstractPtr,
};

impl<'a, M, T, A> AbstractPtr<'a, M, [T], A> {
    pub fn len(self) -> usize {
        non_null_slice_len(self.pointer)
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    pub fn index<I>(self, index: I) -> AbstractPtr<'a, M, <I as SliceIndex<[T]>>::Output, A>
    where
        I: AbstractPtrSliceIndex<[T]> + SliceIndex<[()]> + Clone,
        A: Access,
    {
        unsafe { self.map(|slice| non_null_index(slice, index)) }
    }

    pub fn iter(self) -> impl Iterator<Item = AbstractPtr<'a, M, T, A>>
    where
        A: Access,
    {
        let ptr = non_null_slice_as_mut_ptr(self.pointer);
        let len = self.len();
        (0..len)
            .map(move |i| unsafe { AbstractPtr::new_generic(NonNull::new_unchecked(ptr.add(i))) })
    }

    pub fn copy_into_slice(self, dst: &mut [T])
    where
        M: BulkOps<T>,
        A: Readable,
    {
        let len = self.len();
        assert_eq!(
            len,
            dst.len(),
            "destination and source slices have different lengths"
        );
        unsafe {
            M::memcpy_from(
                dst.as_mut_ptr(),
                non_null_slice_as_mut_ptr(self.pointer),
                len,
            );
        }
    }

    pub fn copy_from_slice(self, src: &[T])
    where
        M: BulkOps<T>,
        A: Writable,
    {
        let len = self.len();
        assert_eq!(
            len,
            src.len(),
            "destination and source slices have different lengths"
        );
        unsafe {
            M::memcpy_into(non_null_slice_as_mut_ptr(self.pointer), src.as_ptr(), len);
        }
    }

    pub fn copy_within(self, src: impl RangeBounds<usize>, dest: usize)
    where
        M: BulkOps<T>,
        A: Readable + Writable,
    {
        let len = self.pointer.len();
        // implementation taken from https://github.com/rust-lang/rust/blob/683d1bcd405727fcc9209f64845bd3b9104878b8/library/core/src/slice/mod.rs#L2726-L2738
        let Range {
            start: src_start,
            end: src_end,
        } = range(src, ..len);
        let count = src_end - src_start;
        assert!(dest <= len - count, "dest is out of bounds");
        // SAFETY: the conditions for `volatile_copy_memory` have all been checked above,
        // as have those for `ptr::add`.
        unsafe {
            M::memmove(
                non_null_slice_as_mut_ptr(self.pointer).add(dest),
                non_null_slice_as_mut_ptr(self.pointer).add(src_start),
                count,
            );
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn split_at(self, mid: usize) -> (AbstractPtr<'a, M, [T], A>, AbstractPtr<'a, M, [T], A>)
    where
        A: Access,
    {
        assert!(mid <= self.pointer.len());
        // SAFETY: `[ptr; mid]` and `[mid; len]` are inside `self`, which
        // fulfills the requirements of `from_raw_parts_mut`.
        unsafe { self.split_at_unchecked(mid) }
    }

    #[allow(clippy::type_complexity)]
    unsafe fn split_at_unchecked(
        self,
        mid: usize,
    ) -> (AbstractPtr<'a, M, [T], A>, AbstractPtr<'a, M, [T], A>)
    where
        A: Access,
    {
        // SAFETY: Caller has to check that `0 <= mid <= self.len()`
        unsafe {
            (
                AbstractPtr::new_generic(non_null_index(self.pointer, ..mid)),
                AbstractPtr::new_generic(non_null_index(self.pointer, mid..)),
            )
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn as_chunks<const N: usize>(
        self,
    ) -> (AbstractPtr<'a, M, [[T; N]], A>, AbstractPtr<'a, M, [T], A>)
    where
        A: Access,
    {
        assert_ne!(N, 0);
        let len = self.pointer.len() / N;
        let (multiple_of_n, remainder) = self.split_at(len * N);
        // SAFETY: We already panicked for zero, and ensured by construction
        // that the length of the subslice is a multiple of N.
        let array_slice = unsafe { multiple_of_n.as_chunks_unchecked() };
        (array_slice, remainder)
    }

    pub unsafe fn as_chunks_unchecked<const N: usize>(self) -> AbstractPtr<'a, M, [[T; N]], A>
    where
        A: Access,
    {
        debug_assert_ne!(N, 0);
        debug_assert_eq!(self.pointer.len() % N, 0);
        let new_len = self.pointer.len() / N;
        // SAFETY: We cast a slice of `new_len * N` elements into
        // a slice of `new_len` many `N` elements chunks.
        let pointer = NonNull::new(ptr::slice_from_raw_parts_mut(
            non_null_slice_as_mut_ptr(self.pointer).cast(),
            new_len,
        ))
        .unwrap();
        unsafe { AbstractPtr::new_generic(pointer) }
    }
}

impl<M, A> AbstractPtr<'_, M, [u8], A> {
    pub fn fill(self, value: u8)
    where
        M: BulkOps<u8>,
        A: Writable,
    {
        unsafe {
            M::memset(
                non_null_slice_as_mut_ptr(self.pointer),
                value,
                non_null_slice_len(self.pointer),
            );
        }
    }
}

/// Methods for converting arrays to slices
impl<'a, M, T, A, const N: usize> AbstractPtr<'a, M, [T; N], A> {
    pub fn as_slice(self) -> AbstractPtr<'a, M, [T], A>
    where
        A: Access,
    {
        unsafe {
            self.map(|array| {
                NonNull::new(ptr::slice_from_raw_parts_mut(array.as_ptr() as *mut T, N)).unwrap()
            })
        }
    }
}
