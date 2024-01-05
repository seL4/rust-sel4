//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT
//

use rustls::unbuffered::{EncodeError, EncryptError};
use rustls::Error as TlsError;

#[derive(Debug)]
pub enum Error<E> {
    TransitError(E),
    ConnectionAborted,
    TlsError(TlsError),
    EncodeError(EncodeError),
    EncryptError(EncryptError),
}

impl<E> From<TlsError> for Error<E> {
    fn from(err: TlsError) -> Self {
        Self::TlsError(err)
    }
}

impl<E> From<EncodeError> for Error<E> {
    fn from(err: EncodeError) -> Self {
        Self::EncodeError(err)
    }
}

impl<E> From<EncryptError> for Error<E> {
    fn from(err: EncryptError) -> Self {
        Self::EncryptError(err)
    }
}
