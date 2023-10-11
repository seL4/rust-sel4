#![no_std]
#![feature(async_fn_in_trait)]
#![feature(return_position_impl_trait_in_trait)]
#![allow(clippy::useless_conversion)]

extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use async_unsync::semaphore::Semaphore;

use sel4_async_block_io::{BlockIO, BlockSize};
use sel4_bounce_buffer_allocator::{AbstractBounceBufferAllocator, BounceBufferAllocator};
use sel4_externally_shared::ExternallySharedRef;
use sel4_shared_ring_buffer::{roles::Provide, RingBuffers};
use sel4_shared_ring_buffer_block_io_types::BlockIORequest;

mod errors;
mod owned;

pub use errors::{Error, ErrorOrUserError, IOError, PeerMisbehaviorError, UserError};
pub use owned::{IssueRequestBuf, OwnedSharedRingBufferBlockIO, PollRequestBuf, RequestBuf};

pub struct SharedRingBufferBlockIO<N, A, F> {
    shared: Rc<RefCell<Inner<N, A, F>>>,
}

struct Inner<N, A, F> {
    owned: OwnedSharedRingBufferBlockIO<Rc<Semaphore>, A, F>,
    block_size: N,
    num_blocks: u64,
}

impl<N, A: AbstractBounceBufferAllocator, F: FnMut()> SharedRingBufferBlockIO<N, A, F> {
    pub fn new(
        block_size: N,
        num_blocks: u64,
        dma_region: ExternallySharedRef<'static, [u8]>,
        bounce_buffer_allocator: BounceBufferAllocator<A>,
        ring_buffers: RingBuffers<'static, Provide, F, BlockIORequest>,
    ) -> Self {
        Self {
            shared: Rc::new(RefCell::new(Inner {
                owned: OwnedSharedRingBufferBlockIO::new(
                    dma_region,
                    bounce_buffer_allocator,
                    ring_buffers,
                ),
                block_size,
                num_blocks,
            })),
        }
    }

    pub fn poll(&self) -> Result<bool, Error> {
        self.shared
            .borrow_mut()
            .owned
            .poll()
            .map_err(ErrorOrUserError::unwrap_error)
    }

    async fn request<'a>(
        &'a self,
        start_block_idx: u64,
        mut request_buf: RequestBuf<'a>,
    ) -> Result<(), Error> {
        let request_index = {
            let sem = self.shared.borrow().owned.slot_set_semaphore().clone();
            let mut reservation = sem.reserve(1).await.unwrap();
            self.shared
                .borrow_mut()
                .owned
                .issue_request(
                    &mut reservation,
                    start_block_idx,
                    &mut request_buf.issue_request_buf(),
                )
                .map_err(ErrorOrUserError::unwrap_error)?
        };
        RequestFuture {
            io: self,
            request_buf,
            request_index,
            poll_returned_ready: false,
        }
        .await
    }
}

impl<N, A, F> Clone for SharedRingBufferBlockIO<N, A, F> {
    fn clone(&self) -> Self {
        Self {
            shared: self.shared.clone(),
        }
    }
}

impl<N: BlockSize + Copy, A: AbstractBounceBufferAllocator, F: FnMut()> BlockIO
    for SharedRingBufferBlockIO<N, A, F>
{
    type Error = Error;

    type BlockSize = N;

    fn block_size(&self) -> Self::BlockSize {
        self.shared.borrow().block_size
    }

    fn num_blocks(&self) -> u64 {
        self.shared.borrow().num_blocks
    }

    async fn read_blocks(&self, start_block_idx: u64, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.request(start_block_idx, RequestBuf::Read { buf })
            .await
    }

    async fn write_blocks(&self, start_block_idx: u64, buf: &[u8]) -> Result<(), Self::Error> {
        self.request(start_block_idx, RequestBuf::Write { buf })
            .await
    }
}

pub struct RequestFuture<'a, N, A: AbstractBounceBufferAllocator, F: FnMut()> {
    io: &'a SharedRingBufferBlockIO<N, A, F>,
    request_buf: RequestBuf<'a>,
    request_index: usize,
    poll_returned_ready: bool,
}

impl<'a, N, A: AbstractBounceBufferAllocator, F: FnMut()> RequestFuture<'a, N, A, F> {
    fn poll_inner<'b>(&'b mut self, cx: &mut Context<'_>) -> Poll<Result<(), Error>>
    where
        'a: 'b,
    {
        assert!(!self.poll_returned_ready);
        let ret = match self
            .io
            .shared
            .borrow_mut()
            .owned
            .poll_request(
                self.request_index,
                &mut self.request_buf.poll_request_buf(),
                Some(cx.waker().clone()),
            )
            .map_err(ErrorOrUserError::unwrap_error)
        {
            Ok(val) => val.map_err(Error::from),
            Err(err) => Poll::Ready(Err(err)),
        };
        if ret.is_ready() {
            self.poll_returned_ready = true;
        }
        ret
    }
}

impl<'a, N, A: AbstractBounceBufferAllocator, F: FnMut()> Future for RequestFuture<'a, N, A, F> {
    type Output = Result<(), Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.poll_inner(cx)
    }
}

impl<'a, N, A: AbstractBounceBufferAllocator, F: FnMut()> Drop for RequestFuture<'a, N, A, F> {
    fn drop(&mut self) {
        if !self.poll_returned_ready {
            self.io
                .shared
                .borrow_mut()
                .owned
                .cancel_request(self.request_index)
                .unwrap();
        }
    }
}
