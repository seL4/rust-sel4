//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::convert::Infallible;
use core::fmt;

use crate::cslot_allocator::CSlotAllocatorError;

#[derive(Debug)]
pub enum CapDLInitializerError {
    CSlotAllocatorError(CSlotAllocatorError),
    #[allow(dead_code)] // false positive
    SeL4Error(sel4::Error),
}

impl From<CSlotAllocatorError> for CapDLInitializerError {
    fn from(err: CSlotAllocatorError) -> Self {
        Self::CSlotAllocatorError(err)
    }
}

impl From<sel4::Error> for CapDLInitializerError {
    fn from(err: sel4::Error) -> Self {
        Self::SeL4Error(err)
    }
}

impl From<Infallible> for CapDLInitializerError {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

impl fmt::Display for CapDLInitializerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO
        write!(f, "{self:?}")
    }
}
