//
// Copyright 2024, Colias Group, LLC
// Copyright (c) 2020 Philipp Oppermann
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

use core::{cmp::Ordering, fmt, hash, marker::PhantomData, ptr::NonNull};

use crate::access::ReadWrite;

mod atomic_operations;
mod macros;
mod operations;
mod slice_operations;

#[must_use]
#[repr(transparent)]
pub struct AbstractPtr<'a, M, T, A = ReadWrite>
where
    T: ?Sized,
{
    pointer: NonNull<T>,
    memory_type: PhantomData<M>,
    reference: PhantomData<&'a T>,
    access: PhantomData<A>,
}

impl<M, T, A> Copy for AbstractPtr<'_, M, T, A> where T: ?Sized {}

impl<M, T, A> Clone for AbstractPtr<'_, M, T, A>
where
    T: ?Sized,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<M, T, A> fmt::Debug for AbstractPtr<'_, M, T, A>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.pointer.as_ptr(), f)
    }
}

impl<M, T, A> fmt::Pointer for AbstractPtr<'_, M, T, A>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.pointer.as_ptr(), f)
    }
}

impl<M, T, A> PartialEq for AbstractPtr<'_, M, T, A>
where
    T: ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.pointer.as_ptr(), other.pointer.as_ptr())
    }
}

impl<M, T, A> Eq for AbstractPtr<'_, M, T, A> where T: ?Sized {}

impl<M, T, A> PartialOrd for AbstractPtr<'_, M, T, A>
where
    T: ?Sized,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<M, T, A> Ord for AbstractPtr<'_, M, T, A>
where
    T: ?Sized,
{
    fn cmp(&self, other: &Self) -> Ordering {
        #[allow(ambiguous_wide_pointer_comparisons)]
        Ord::cmp(&self.pointer.as_ptr(), &other.pointer.as_ptr())
    }
}

impl<M, T, A> hash::Hash for AbstractPtr<'_, M, T, A>
where
    T: ?Sized,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.pointer.as_ptr().hash(state);
    }
}
