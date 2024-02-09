//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use core::cell::{Ref, RefCell, RefMut};
use core::fmt;
use core::sync::atomic::{AtomicIsize, Ordering};

pub(crate) trait Token {
    const INIT: Self;

    type Borrow<'a>
    where
        Self: 'a;

    type BorrowMut<'a>
    where
        Self: 'a;

    fn try_borrow<'a>(&'a self) -> Result<Self::Borrow<'a>, BorrowError>;

    fn try_borrow_mut<'a>(&'a self) -> Result<Self::BorrowMut<'a>, BorrowMutError>;
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

#[allow(dead_code)]
pub(crate) struct UnsyncToken(RefCell<()>);

impl Token for UnsyncToken {
    const INIT: Self = Self(RefCell::new(()));

    type Borrow<'a> = Ref<'a, ()>;
    type BorrowMut<'a> = RefMut<'a, ()>;

    fn try_borrow<'a>(&'a self) -> Result<Self::Borrow<'a>, BorrowError> {
        self.0.try_borrow().map_err(|_| BorrowError::new())
    }

    fn try_borrow_mut<'a>(&'a self) -> Result<Self::BorrowMut<'a>, BorrowMutError> {
        self.0.try_borrow_mut().map_err(|_| BorrowMutError::new())
    }
}

#[allow(dead_code)]
pub(crate) struct SyncToken(BorrowFlag);

type BorrowFlag = AtomicIsize;

pub(crate) struct SyncTokenBorrow<'a>(&'a BorrowFlag);

impl<'a> Drop for SyncTokenBorrow<'a> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Release);
    }
}

pub(crate) struct SyncTokenBorrowMut<'a>(&'a BorrowFlag);

impl<'a> Drop for SyncTokenBorrowMut<'a> {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::Release);
    }
}

impl Token for SyncToken {
    const INIT: Self = Self(AtomicIsize::new(0));

    type Borrow<'a> = SyncTokenBorrow<'a>;
    type BorrowMut<'a> = SyncTokenBorrowMut<'a>;

    fn try_borrow<'a>(&'a self) -> Result<Self::Borrow<'a>, BorrowError> {
        let mut current = self.0.load(Ordering::SeqCst);
        loop {
            if 0 <= current && current < isize::MAX {
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

    fn try_borrow_mut<'a>(&'a self) -> Result<Self::BorrowMut<'a>, BorrowMutError> {
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
