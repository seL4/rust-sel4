//
// Copyright 2024, Colias Group, LLC
// Copyright (c) 2020 Philipp Oppermann
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

use core::{marker::PhantomData, ptr::NonNull};

use crate::{
    AbstractPtr,
    access::{Access, ReadOnly, ReadWrite, Readable, RestrictAccess, Writable, WriteOnly},
    memory_type::UnitaryOps,
};

/// Constructor functions.
impl<'a, M, T> AbstractPtr<'a, M, T>
where
    T: ?Sized,
{
    pub unsafe fn new(pointer: NonNull<T>) -> AbstractPtr<'a, M, T, ReadWrite> {
        unsafe { AbstractPtr::new_restricted(ReadWrite, pointer) }
    }

    pub const unsafe fn new_read_only(pointer: NonNull<T>) -> AbstractPtr<'a, M, T, ReadOnly> {
        unsafe { Self::new_restricted(ReadOnly, pointer) }
    }

    pub const unsafe fn new_restricted<A>(
        access: A,
        pointer: NonNull<T>,
    ) -> AbstractPtr<'a, M, T, A>
    where
        A: Access,
    {
        let _ = access;
        unsafe { Self::new_generic(pointer) }
    }

    pub(super) const unsafe fn new_generic<A>(pointer: NonNull<T>) -> AbstractPtr<'a, M, T, A> {
        AbstractPtr {
            pointer,
            memory_type: PhantomData,
            reference: PhantomData,
            access: PhantomData,
        }
    }
}

impl<'a, M, T, A> AbstractPtr<'a, M, T, A>
where
    T: ?Sized,
{
    #[must_use]
    pub fn read(self) -> T
    where
        M: UnitaryOps<T>,
        T: Sized,
        A: Readable,
    {
        unsafe { M::read(self.pointer.as_ptr()) }
    }

    pub fn write(self, value: T)
    where
        M: UnitaryOps<T>,
        T: Sized,
        A: Writable,
    {
        unsafe { M::write(self.pointer.as_ptr(), value) };
    }

    pub fn update<F>(self, f: F)
    where
        M: UnitaryOps<T>,
        T: Sized,
        A: Readable + Writable,
        F: FnOnce(T) -> T,
    {
        let new = f(self.read());
        self.write(new);
    }

    #[must_use]
    pub fn as_raw_ptr(self) -> NonNull<T> {
        self.pointer
    }

    pub unsafe fn map<F, U>(self, f: F) -> AbstractPtr<'a, M, U, A>
    where
        F: FnOnce(NonNull<T>) -> NonNull<U>,
        A: Access,
        U: ?Sized,
    {
        unsafe { AbstractPtr::new_restricted(A::default(), f(self.pointer)) }
    }
}

/// Methods for restricting access.
impl<'a, M, T, A> AbstractPtr<'a, M, T, A>
where
    T: ?Sized,
{
    pub fn restrict<To>(self) -> AbstractPtr<'a, M, T, A::Restricted>
    where
        A: RestrictAccess<To>,
    {
        unsafe { AbstractPtr::new_restricted(Default::default(), self.pointer) }
    }
}

/// Methods for restricting access.
impl<'a, M, T> AbstractPtr<'a, M, T, ReadWrite>
where
    T: ?Sized,
{
    pub fn read_only(self) -> AbstractPtr<'a, M, T, ReadOnly> {
        self.restrict()
    }

    pub fn write_only(self) -> AbstractPtr<'a, M, T, WriteOnly> {
        self.restrict()
    }
}
