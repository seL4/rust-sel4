use core::{fmt, mem, result};

use crate::sys;

pub type Result<T> = result::Result<T, Error>;

#[repr(u32)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Error {
    InvalidArgument = sys::seL4_Error::seL4_InvalidArgument,
    InvalidCapability = sys::seL4_Error::seL4_InvalidCapability,
    IllegalOperation = sys::seL4_Error::seL4_IllegalOperation,
    RangeError = sys::seL4_Error::seL4_RangeError,
    AlignmentError = sys::seL4_Error::seL4_AlignmentError,
    FailedLookup = sys::seL4_Error::seL4_FailedLookup,
    TruncatedMessage = sys::seL4_Error::seL4_TruncatedMessage,
    DeleteFirst = sys::seL4_Error::seL4_DeleteFirst,
    RevokeFirst = sys::seL4_Error::seL4_RevokeFirst,
    NotEnoughMemory = sys::seL4_Error::seL4_NotEnoughMemory,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "seL4_Error: {:?}", self)
    }
}

impl Error {
    pub const fn into_sys(self) -> sys::seL4_Error::Type {
        self as sys::seL4_Error::Type
    }

    pub fn from_sys(err: sys::seL4_Error::Type) -> Option<Self> {
        match err {
            sys::seL4_Error::seL4_NoError => None,
            err if err < sys::seL4_Error::seL4_NumErrors => Some(unsafe { mem::transmute(err) }),
            _ => panic!("invalid seL4_Error: {}", err),
        }
    }

    pub(crate) fn wrap(err: sys::seL4_Error::Type) -> Result<()> {
        Self::or(err, ())
    }

    pub(crate) fn or<T>(err: sys::seL4_Error::Type, value: T) -> Result<T> {
        match Self::from_sys(err) {
            None => Ok(value),
            Some(err) => Err(err),
        }
    }
}

#[allow(dead_code)]
#[allow(non_upper_case_globals)]
mod __assertions {
    use super::*;

    const __assert_all_errors_accounted_for: () = {
        assert!(mem::variant_count::<Error>() == sys::seL4_Error::seL4_NumErrors as usize - 1);
    };
}
