//
// Copyright 2024, Colias Group, LLC
// Copyright 2024, Embedded devices Working Group
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

// TODO use Pin

#![no_std]

use core::future::poll_fn;
use core::pin::Pin;
use core::task::{Context, Poll};

pub use embedded_io_async::{Error, ErrorKind, ErrorType};

pub trait Read: ErrorType {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>>;

    // // //

    #[allow(async_fn_in_trait)]
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>
    where
        Self: Unpin,
    {
        let mut pin = Pin::new(self);
        poll_fn(move |cx| pin.as_mut().poll_read(cx, buf)).await
    }

    #[allow(async_fn_in_trait)]
    async fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<(), ReadExactError<Self::Error>>
    where
        Self: Unpin,
    {
        while !buf.is_empty() {
            match self.read(buf).await {
                Ok(0) => break,
                Ok(n) => buf = &mut buf[n..],
                Err(e) => return Err(ReadExactError::Other(e)),
            }
        }
        if buf.is_empty() {
            Ok(())
        } else {
            Err(ReadExactError::UnexpectedEof)
        }
    }
}

pub trait Write: ErrorType {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>>;

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    // // //

    #[allow(async_fn_in_trait)]
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error>
    where
        Self: Unpin,
    {
        let mut pin = Pin::new(self);
        poll_fn(|cx| pin.as_mut().poll_write(cx, buf)).await
    }

    #[allow(async_fn_in_trait)]
    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error>
    where
        Self: Unpin,
    {
        let mut buf = buf;
        while !buf.is_empty() {
            match self.write(buf).await {
                Ok(0) => panic!("write() returned Ok(0)"),
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    #[allow(async_fn_in_trait)]
    async fn flush(&mut self) -> Result<(), Self::Error>
    where
        Self: Unpin,
    {
        let mut pin = Pin::new(self);
        poll_fn(|cx| pin.as_mut().poll_flush(cx)).await
    }
}

/// Error returned by [`Read::read_exact`]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ReadExactError<E> {
    /// An EOF error was encountered before reading the exact amount of requested bytes.
    UnexpectedEof,
    /// Error returned by the inner Read.
    Other(E),
}

impl<E> From<E> for ReadExactError<E> {
    fn from(err: E) -> Self {
        Self::Other(err)
    }
}
