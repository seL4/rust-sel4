//
// Copyright 2024, Colias Group, LLC
// Copyright (c) 2020 Philipp Oppermann
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

use crate::{
    abstract_ptr::AbstractPtr,
    access::{Access, Copyable, ReadOnly, ReadWrite, RestrictAccess, WriteOnly},
};
use core::{cmp::Ordering, fmt, hash, marker::PhantomData, ptr::NonNull};

#[must_use]
#[repr(transparent)]
pub struct AbstractRef<'a, M, T, A = ReadWrite>
where
    T: ?Sized,
{
    pointer: NonNull<T>,
    memory_type: PhantomData<M>,
    reference: PhantomData<&'a T>,
    access: PhantomData<A>,
}

impl<'a, M, T> AbstractRef<'a, M, T>
where
    T: ?Sized,
{
    pub unsafe fn new(pointer: NonNull<T>) -> Self {
        unsafe { AbstractRef::new_restricted(ReadWrite, pointer) }
    }

    pub const unsafe fn new_read_only(pointer: NonNull<T>) -> AbstractRef<'a, M, T, ReadOnly> {
        unsafe { Self::new_restricted(ReadOnly, pointer) }
    }

    pub const unsafe fn new_restricted<A>(
        access: A,
        pointer: NonNull<T>,
    ) -> AbstractRef<'a, M, T, A>
    where
        A: Access,
    {
        let _ = access;
        unsafe { Self::new_generic(pointer) }
    }

    pub fn from_ref(reference: &'a T) -> AbstractRef<'a, M, T, ReadOnly>
    where
        T: 'a,
    {
        unsafe { AbstractRef::new_restricted(ReadOnly, reference.into()) }
    }

    pub fn from_mut_ref(reference: &'a mut T) -> Self
    where
        T: 'a,
    {
        unsafe { AbstractRef::new(reference.into()) }
    }

    const unsafe fn new_generic<A>(pointer: NonNull<T>) -> AbstractRef<'a, M, T, A> {
        AbstractRef {
            pointer,
            memory_type: PhantomData,
            reference: PhantomData,
            access: PhantomData,
        }
    }
}

impl<'a, M, T, A> AbstractRef<'a, M, T, A>
where
    T: ?Sized,
{
    pub fn borrow(&self) -> AbstractRef<'_, M, T, A::Restricted>
    where
        A: RestrictAccess<ReadOnly>,
    {
        unsafe { AbstractRef::new_restricted(Default::default(), self.pointer) }
    }

    pub fn borrow_mut(&mut self) -> AbstractRef<'_, M, T, A>
    where
        A: Access,
    {
        unsafe { AbstractRef::new_restricted(Default::default(), self.pointer) }
    }

    pub fn as_ptr(&self) -> AbstractPtr<'_, M, T, A::Restricted>
    where
        A: RestrictAccess<ReadOnly>,
    {
        unsafe { AbstractPtr::new_restricted(Default::default(), self.pointer) }
    }

    pub fn as_mut_ptr(&mut self) -> AbstractPtr<'_, M, T, A>
    where
        A: Access,
    {
        unsafe { AbstractPtr::new_restricted(Default::default(), self.pointer) }
    }

    pub fn into_ptr(self) -> AbstractPtr<'a, M, T, A>
    where
        A: Access,
    {
        unsafe { AbstractPtr::new_restricted(Default::default(), self.pointer) }
    }
}

impl<'a, M, T, A> AbstractRef<'a, M, T, A>
where
    T: ?Sized,
{
    pub fn restrict<To>(self) -> AbstractRef<'a, M, T, A::Restricted>
    where
        A: RestrictAccess<To>,
    {
        unsafe { AbstractRef::new_restricted(Default::default(), self.pointer) }
    }
}

impl<'a, M, T> AbstractRef<'a, M, T, ReadWrite>
where
    T: ?Sized,
{
    pub fn read_only(self) -> AbstractRef<'a, M, T, ReadOnly> {
        self.restrict()
    }

    pub fn write_only(self) -> AbstractRef<'a, M, T, WriteOnly> {
        self.restrict()
    }
}

impl<M, T, A> Clone for AbstractRef<'_, M, T, A>
where
    T: ?Sized,
    A: Access + Copyable,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<M, T, A> Copy for AbstractRef<'_, M, T, A>
where
    T: ?Sized,
    A: Access + Copyable,
{
}

unsafe impl<M, T, A> Send for AbstractRef<'_, M, T, A> where T: Sync + ?Sized {}
unsafe impl<M, T, A> Sync for AbstractRef<'_, M, T, A> where T: Sync + ?Sized {}

impl<M, T, A> fmt::Debug for AbstractRef<'_, M, T, A>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.pointer.as_ptr(), f)
    }
}

impl<M, T, A> fmt::Pointer for AbstractRef<'_, M, T, A>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.pointer.as_ptr(), f)
    }
}

impl<M, T, A> PartialEq for AbstractRef<'_, M, T, A>
where
    T: ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.pointer.as_ptr(), other.pointer.as_ptr())
    }
}

impl<M, T, A> Eq for AbstractRef<'_, M, T, A> where T: ?Sized {}

impl<M, T, A> PartialOrd for AbstractRef<'_, M, T, A>
where
    T: ?Sized,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<M, T, A> Ord for AbstractRef<'_, M, T, A>
where
    T: ?Sized,
{
    fn cmp(&self, other: &Self) -> Ordering {
        #[allow(ambiguous_wide_pointer_comparisons)]
        Ord::cmp(&self.pointer.as_ptr(), &other.pointer.as_ptr())
    }
}

impl<M, T, A> hash::Hash for AbstractRef<'_, M, T, A>
where
    T: ?Sized,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.pointer.as_ptr().hash(state);
    }
}
