use crate::CSlotAllocatorError;
use capdl_types::*;
use core::convert::Infallible;
use core::fmt;
use core::num::TryFromIntError;

#[derive(Debug)]
pub enum CapDLLoaderError {
    CSlotAllocatorError(CSlotAllocatorError),
    SeL4Error(sel4::Error),
    TryFromObjectError(TryFromObjectError),
    TryFromCapError(TryFromCapError),
    TryFromIntError(TryFromIntError),
}

impl From<CSlotAllocatorError> for CapDLLoaderError {
    fn from(err: CSlotAllocatorError) -> Self {
        Self::CSlotAllocatorError(err)
    }
}

impl From<sel4::Error> for CapDLLoaderError {
    fn from(err: sel4::Error) -> Self {
        Self::SeL4Error(err)
    }
}

impl From<TryFromObjectError> for CapDLLoaderError {
    fn from(err: TryFromObjectError) -> Self {
        Self::TryFromObjectError(err)
    }
}

impl From<TryFromCapError> for CapDLLoaderError {
    fn from(err: TryFromCapError) -> Self {
        Self::TryFromCapError(err)
    }
}

impl From<Infallible> for CapDLLoaderError {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

impl From<TryFromIntError> for CapDLLoaderError {
    fn from(err: TryFromIntError) -> Self {
        Self::TryFromIntError(err)
    }
}

impl fmt::Display for CapDLLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO
        write!(f, "{:?}", self)
    }
}
