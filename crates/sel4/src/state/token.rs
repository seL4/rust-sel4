//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use core::cell::{Ref, RefCell, RefMut, UnsafeCell};
use core::fmt;
use core::sync::atomic::{AtomicIsize, Ordering};

pub(crate) struct TokenCell<K, A> {
    token: K,
    accessor: A,
}

pub(crate) trait Accessor<T> {
    fn with<F, U>(&self, f: F) -> U
    where
        F: FnOnce(&UnsafeCell<T>) -> U;
}

impl<K: Token, A> TokenCell<K, A> {
    pub(crate) const unsafe fn new(accessor: A) -> Self {
        Self {
            token: K::INIT,
            accessor,
        }
    }

    pub(crate) fn try_with<F, T, U>(&self, f: F) -> U
    where
        A: Accessor<T>,
        F: FnOnce(Result<&T, BorrowError>) -> U,
    {
        let access = || {
            self.accessor
                .with(|cell| unsafe { cell.get().as_ref().unwrap() })
        };
        self.token.try_with(access, f)
    }

    pub(crate) fn try_with_mut<F, T, U>(&self, f: F) -> U
    where
        A: Accessor<T>,
        F: FnOnce(Result<&mut T, BorrowMutError>) -> U,
    {
        let access = || {
            self.accessor
                .with(|cell| unsafe { cell.get().as_mut().unwrap() })
        };
        self.token.try_with_mut(access, f)
    }
}

pub(crate) trait Token {
    const INIT: Self;

    type Borrow<'a>
    where
        Self: 'a;

    type BorrowMut<'a>
    where
        Self: 'a;

    fn try_borrow(&self) -> Result<Self::Borrow<'_>, BorrowError>;

    fn try_borrow_mut(&self) -> Result<Self::BorrowMut<'_>, BorrowMutError>;

    fn try_with<F, G, T, U>(&self, access_resource: G, f: F) -> T
    where
        F: FnOnce(Result<U, BorrowError>) -> T,
        G: FnOnce() -> U,
    {
        let (_tok, r) = take_ok(self.try_borrow());
        f(r.map(|_| access_resource()))
    }

    fn try_with_mut<F, G, T, U>(&self, access_resource: G, f: F) -> T
    where
        F: FnOnce(Result<U, BorrowMutError>) -> T,
        G: FnOnce() -> U,
    {
        let (_tok, r) = take_ok(self.try_borrow_mut());
        f(r.map(|_| access_resource()))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BorrowError(());

impl BorrowError {
    pub(crate) fn new() -> Self {
        Self(())
    }
}

impl fmt::Display for BorrowError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "already mutably borrowed")
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BorrowMutError(());

impl BorrowMutError {
    pub(crate) fn new() -> Self {
        Self(())
    }
}

impl fmt::Display for BorrowMutError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "already borrowed")
    }
}

fn take_ok<T, E>(r: Result<T, E>) -> (Option<T>, Result<(), E>) {
    match r {
        Ok(ok) => (Some(ok), Ok(())),
        Err(err) => (None, Err(err)),
    }
}

#[allow(dead_code)]
pub(crate) struct UnsyncToken(RefCell<()>);

impl Token for UnsyncToken {
    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = Self(RefCell::new(()));

    type Borrow<'a> = Ref<'a, ()>;
    type BorrowMut<'a> = RefMut<'a, ()>;

    fn try_borrow(&self) -> Result<Self::Borrow<'_>, BorrowError> {
        self.0.try_borrow().map_err(|_| BorrowError::new())
    }

    fn try_borrow_mut(&self) -> Result<Self::BorrowMut<'_>, BorrowMutError> {
        self.0.try_borrow_mut().map_err(|_| BorrowMutError::new())
    }
}

#[allow(dead_code)]
pub(crate) struct SyncToken(BorrowFlag);

type BorrowFlag = AtomicIsize;

pub(crate) struct SyncTokenBorrow<'a>(&'a BorrowFlag);

impl Drop for SyncTokenBorrow<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Release);
    }
}

pub(crate) struct SyncTokenBorrowMut<'a>(&'a BorrowFlag);

impl Drop for SyncTokenBorrowMut<'_> {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::Release);
    }
}

impl Token for SyncToken {
    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = Self(AtomicIsize::new(0));

    type Borrow<'a> = SyncTokenBorrow<'a>;
    type BorrowMut<'a> = SyncTokenBorrowMut<'a>;

    fn try_borrow(&self) -> Result<Self::Borrow<'_>, BorrowError> {
        let mut current = self.0.load(Ordering::SeqCst);
        loop {
            if (0..isize::MAX).contains(&current) {
                match self.0.compare_exchange(
                    current,
                    current + 1,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return Ok(SyncTokenBorrow(&self.0)),
                    Err(actual_current) => {
                        current = actual_current;
                    }
                }
            } else {
                return Err(BorrowError::new());
            }
        }
    }

    fn try_borrow_mut(&self) -> Result<Self::BorrowMut<'_>, BorrowMutError> {
        let mut current = self.0.load(Ordering::SeqCst);
        loop {
            if current == 0 {
                match self.0.compare_exchange(
                    current,
                    current - 1,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return Ok(SyncTokenBorrowMut(&self.0)),
                    Err(actual_current) => {
                        current = actual_current;
                    }
                }
            } else {
                return Err(BorrowMutError::new());
            }
        }
    }
}
