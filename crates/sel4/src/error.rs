//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2020 Arm Limited
//
// SPDX-License-Identifier: MIT
//

use core::{fmt, result};

use crate::sys;

/// Alias for `Result<_, Error>`.
pub type Result<T> = result::Result<T, Error>;

/// Corresponds to `seL4_Error`.
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
        write!(f, "seL4_Error: {self:?}")
    }
}

impl Error {
    pub const fn into_sys(self) -> sys::seL4_Error::Type {
        self as sys::seL4_Error::Type
    }

    pub fn from_sys(err: sys::seL4_Error::Type) -> Option<Self> {
        match err {
            sys::seL4_Error::seL4_NoError => None,
            sys::seL4_Error::seL4_InvalidArgument => Some(Self::InvalidArgument),
            sys::seL4_Error::seL4_InvalidCapability => Some(Self::InvalidCapability),
            sys::seL4_Error::seL4_IllegalOperation => Some(Self::IllegalOperation),
            sys::seL4_Error::seL4_RangeError => Some(Self::RangeError),
            sys::seL4_Error::seL4_AlignmentError => Some(Self::AlignmentError),
            sys::seL4_Error::seL4_FailedLookup => Some(Self::FailedLookup),
            sys::seL4_Error::seL4_TruncatedMessage => Some(Self::TruncatedMessage),
            sys::seL4_Error::seL4_DeleteFirst => Some(Self::DeleteFirst),
            sys::seL4_Error::seL4_RevokeFirst => Some(Self::RevokeFirst),
            sys::seL4_Error::seL4_NotEnoughMemory => Some(Self::NotEnoughMemory),
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

// TODO no way to run this test
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn all_sys_errors_are_accounted_for() {
        for i in 0..sys::seL4_Error::seL4_NumErrors {
            if i != sys::seL4_Error::seL4_NoError {
                assert!(Error::from_sys(i).is_some())
            }
        }
    }
}

// NOTE(rustc_wishlist)
// Use this static test once #![feature(variant_count)] stabilizes.
// With this test, consider replacing `Error::from_sys` with mem::transmute-based implementation.
//
// #[allow(dead_code)]
// #[allow(non_upper_case_globals)]
// mod __assertions {
//     use super::*;
//
//     const __assert_all_errors_accounted_for: () = {
//         assert!(mem::variant_count::<Error>() == sys::seL4_Error::seL4_NumErrors as usize - 1);
//     };
// }
