//
// Copyright 2024, Colias Group, LLC
// Copyright 2024, Embedded devices Working Group
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

// TODO use Pin

#![no_std]

use core::future::{Future, poll_fn};
use core::pin::{Pin, pin};
use core::task::{Context, Poll};

use embedded_io_async as eio;

pub use embedded_io_async::{Error, ErrorKind, ErrorType};

pub trait Read: ErrorType {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>>;
}

pub trait Write: ErrorType {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>>;

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl<T: Read + Unpin> Read for &mut T {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        T::poll_read(Pin::new(*Pin::into_inner(self)), cx, buf)
    }
}

impl<T: Write + Unpin> Write for &mut T {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        T::poll_write(Pin::new(*Pin::into_inner(self)), cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        T::poll_flush(Pin::new(*Pin::into_inner(self)), cx)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct EmbeddedIOAsyncAdapter<T>(pub T);

impl<T: ErrorType> ErrorType for EmbeddedIOAsyncAdapter<T> {
    type Error = T::Error;
}

impl<T> eio::Read for EmbeddedIOAsyncAdapter<T>
where
    T: Read + Unpin,
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        poll_fn(|cx| Pin::new(&mut self.0).poll_read(cx, buf)).await
    }
}

impl<T> eio::Write for EmbeddedIOAsyncAdapter<T>
where
    T: Write + Unpin,
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        poll_fn(|cx| Pin::new(&mut self.0).poll_write(cx, buf)).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        poll_fn(|cx| Pin::new(&mut self.0).poll_flush(cx)).await
    }
}

pub trait ReadCancelSafe: eio::Read {}
pub trait FlushCancelSafe: eio::Write {}
pub trait WriteCancelSafe: FlushCancelSafe {}

impl<T: Read + Unpin> ReadCancelSafe for EmbeddedIOAsyncAdapter<T> {}
impl<T: Write + Unpin> FlushCancelSafe for EmbeddedIOAsyncAdapter<T> {}
impl<T: Write + Unpin> WriteCancelSafe for EmbeddedIOAsyncAdapter<T> {}

impl<T> Read for EmbeddedIOAsyncAdapter<T>
where
    T: eio::Read + ReadCancelSafe + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        pin!(self.0.read(buf)).poll(cx)
    }
}

impl<T> Write for EmbeddedIOAsyncAdapter<T>
where
    T: eio::Write + WriteCancelSafe + Unpin,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        pin!(self.0.write(buf)).poll(cx)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        pin!(self.0.flush()).poll(cx)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct EmbeddedIOAsyncAdapterUsingReady<T>(pub T);

impl<T: ErrorType> ErrorType for EmbeddedIOAsyncAdapterUsingReady<T> {
    type Error = T::Error;
}

impl<T> Read for EmbeddedIOAsyncAdapterUsingReady<T>
where
    T: eio::Read + eio::ReadReady + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        if self.0.read_ready()? {
            match pin!(self.0.read(buf)).poll(cx) {
                Poll::Ready(r) => Poll::Ready(r),
                Poll::Pending => unreachable!(),
            }
        } else {
            Poll::Pending
        }
    }
}

impl<T> Write for EmbeddedIOAsyncAdapterUsingReady<T>
where
    T: eio::Write + eio::WriteReady + FlushCancelSafe + Unpin,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        if self.0.write_ready()? {
            match pin!(self.0.write(buf)).poll(cx) {
                Poll::Ready(r) => Poll::Ready(r),
                Poll::Pending => unreachable!(),
            }
        } else {
            Poll::Pending
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        pin!(self.0.flush()).poll(cx)
    }
}
