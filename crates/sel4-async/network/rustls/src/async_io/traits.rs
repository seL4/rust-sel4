//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: Apache-2.0 OR ISC OR MIT
//

// TODO use Pin

use alloc::boxed::Box;
use core::pin::Pin;
use core::task::{Context, Poll};

use futures::future;

// TODO remove after bumping rust toolchain
use async_trait::async_trait;

pub trait AsyncIO {
    type Error;

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>>;

    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>>;

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
}

#[derive(Copy, Clone, Debug)]
pub enum ClosedError<E> {
    Other(E),
    Closed,
}

impl<E> From<E> for ClosedError<E> {
    fn from(err: E) -> Self {
        Self::Other(err)
    }
}

#[async_trait(?Send)]
pub trait AsyncIOExt: AsyncIO {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>
    where
        Self: Unpin,
    {
        let mut pin = Pin::new(self);
        future::poll_fn(move |cx| pin.as_mut().poll_read(cx, buf)).await
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), ClosedError<Self::Error>>
    where
        Self: Unpin,
    {
        let mut pos = 0;
        while pos < buf.len() {
            let n = self.read(&mut buf[pos..]).await?;
            if n == 0 {
                return Err(ClosedError::Closed);
            }
            pos += n;
        }
        assert_eq!(pos, buf.len());
        Ok(())
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error>
    where
        Self: Unpin,
    {
        let mut pin = Pin::new(self);
        future::poll_fn(|cx| pin.as_mut().poll_write(cx, buf)).await
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<(), ClosedError<Self::Error>>
    where
        Self: Unpin,
    {
        let mut pos = 0;
        while pos < buf.len() {
            let n = self.write(&buf[pos..]).await?;
            if n == 0 {
                return Err(ClosedError::Closed);
            }
            pos += n;
        }
        assert_eq!(pos, buf.len());
        Ok(())
    }

    async fn flush(&mut self) -> Result<(), Self::Error>
    where
        Self: Unpin,
    {
        let mut pin = Pin::new(self);
        future::poll_fn(|cx| pin.as_mut().poll_flush(cx)).await
    }
}

impl<T: AsyncIO + ?Sized> AsyncIOExt for T {}
