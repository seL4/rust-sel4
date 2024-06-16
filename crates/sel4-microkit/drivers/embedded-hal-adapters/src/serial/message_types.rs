//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Galois, Inc.
//
// SPDX-License-Identifier: BSD-2-Clause
//

use embedded_hal_nb::nb;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum NonBlocking<T> {
    Ready(T),
    WouldBlock,
}

impl<T> NonBlocking<T> {
    pub(crate) fn from_nb_result<E>(r: nb::Result<T, E>) -> Result<Self, E> {
        match r {
            Ok(v) => Ok(Self::Ready(v)),
            Err(nb::Error::WouldBlock) => Ok(Self::WouldBlock),
            Err(nb::Error::Other(err)) => Err(err),
        }
    }
}

impl<T> From<Option<T>> for NonBlocking<T> {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(v) => NonBlocking::Ready(v),
            None => NonBlocking::WouldBlock,
        }
    }
}

impl<T, E> From<NonBlocking<T>> for nb::Result<T, E> {
    fn from(v: NonBlocking<T>) -> Self {
        match v {
            NonBlocking::Ready(v) => Ok(v),
            NonBlocking::WouldBlock => Err(nb::Error::WouldBlock),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Request {
    Read,
    Write(u8),
    Flush,
}

pub(crate) type Response = Result<SuccessResponse, ErrorResponse>;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum SuccessResponse {
    Read(NonBlocking<u8>),
    Write(NonBlocking<()>),
    Flush(NonBlocking<()>),
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ErrorResponse {
    WriteError,
    FlushError,
}
