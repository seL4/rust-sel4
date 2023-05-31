use crate::CSlotAllocatorError;
use capdl_types::*;
use core::convert::Infallible;
use core::fmt;
use core::num::TryFromIntError;

#[derive(Debug)]
pub enum CapDLInitializerError {
    CSlotAllocatorError(CSlotAllocatorError),
    SeL4Error(sel4::Error),
    TryFromObjectError(TryFromObjectError),
    TryFromCapError(TryFromCapError),
    TryFromIntError(TryFromIntError),
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

impl From<TryFromObjectError> for CapDLInitializerError {
    fn from(err: TryFromObjectError) -> Self {
        Self::TryFromObjectError(err)
    }
}

impl From<TryFromCapError> for CapDLInitializerError {
    fn from(err: TryFromCapError) -> Self {
        Self::TryFromCapError(err)
    }
}

impl From<Infallible> for CapDLInitializerError {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

impl From<TryFromIntError> for CapDLInitializerError {
    fn from(err: TryFromIntError) -> Self {
        Self::TryFromIntError(err)
    }
}

impl fmt::Display for CapDLInitializerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO
        write!(f, "{self:?}")
    }
}
