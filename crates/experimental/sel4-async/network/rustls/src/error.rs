//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT
//

use core::fmt::Debug;

use rustls::Error as TlsError;
use rustls::unbuffered::{EncodeError, EncryptError};
use thiserror::Error;

use sel4_async_io::{Error as AsyncIOError, ErrorKind};

#[derive(Debug, Error)]
pub enum Error<E> {
    #[error("transit error: {0}")]
    TransitError(E),
    #[error("connection aborted")]
    ConnectionAborted,
    #[error("tls error: {0}")]
    TlsError(TlsError),
    #[error("encode error: {0}")]
    EncodeError(EncodeError),
    #[error("encrypt error: {0}")]
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

impl<E: Debug + core::error::Error> AsyncIOError for Error<E> {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}
