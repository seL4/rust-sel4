//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(async_fn_in_trait)]
#![feature(return_position_impl_trait_in_trait)]
#![allow(clippy::useless_conversion)]

extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

use async_unsync::semaphore::Semaphore;

use sel4_async_block_io::{access::Access, BlockIO, BlockIOLayout, BlockSize, Operation};
use sel4_bounce_buffer_allocator::{AbstractBounceBufferAllocator, BounceBufferAllocator};
use sel4_externally_shared::ExternallySharedRef;
use sel4_shared_ring_buffer::{roles::Provide, RingBuffers};
use sel4_shared_ring_buffer_block_io_types::BlockIORequest;

mod errors;
mod owned;

pub use errors::{Error, ErrorOrUserError, IOError, PeerMisbehaviorError, UserError};
pub use owned::{IssueRequestBuf, OwnedSharedRingBufferBlockIO, PollRequestBuf};

pub struct SharedRingBufferBlockIO<N, P, A, F> {
    shared: Rc<RefCell<Inner<N, P, A, F>>>,
}

struct Inner<N, P, A, F> {
    owned: OwnedSharedRingBufferBlockIO<Rc<Semaphore>, A, F>,
    block_size: N,
    num_blocks: u64,
    _phantom: PhantomData<P>,
}

impl<N, P: Access, A: AbstractBounceBufferAllocator, F: FnMut()>
    SharedRingBufferBlockIO<N, P, A, F>
{
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
                _phantom: PhantomData,
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
        operation: Operation<'a, P>,
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
                    &mut IssueRequestBuf::new(&operation),
                )
                .map_err(ErrorOrUserError::unwrap_error)?
        };
        RequestFuture {
            io: self,
            operation,
            request_index,
            poll_returned_ready: false,
        }
        .await
    }
}

impl<N, P, A, F> Clone for SharedRingBufferBlockIO<N, P, A, F> {
    fn clone(&self) -> Self {
        Self {
            shared: self.shared.clone(),
        }
    }
}

impl<N: BlockSize + Copy, P: Access, A: AbstractBounceBufferAllocator, F: FnMut()> BlockIOLayout
    for SharedRingBufferBlockIO<N, P, A, F>
{
    type Error = Error;

    type BlockSize = N;

    fn block_size(&self) -> Self::BlockSize {
        self.shared.borrow().block_size
    }

    fn num_blocks(&self) -> u64 {
        self.shared.borrow().num_blocks
    }
}

impl<N: BlockSize + Copy, P: Access, A: AbstractBounceBufferAllocator, F: FnMut()> BlockIO<P>
    for SharedRingBufferBlockIO<N, P, A, F>
{
    async fn read_or_write_blocks(
        &self,
        start_block_idx: u64,
        operation: Operation<'_, P>,
    ) -> Result<(), Self::Error> {
        self.request(start_block_idx, operation).await
    }
}

pub struct RequestFuture<'a, N, P: Access, A: AbstractBounceBufferAllocator, F: FnMut()> {
    io: &'a SharedRingBufferBlockIO<N, P, A, F>,
    operation: Operation<'a, P>,
    request_index: usize,
    poll_returned_ready: bool,
}

impl<'a, N, P: Access, A: AbstractBounceBufferAllocator, F: FnMut()> RequestFuture<'a, N, P, A, F> {
    fn poll_inner<'b>(&'b mut self, cx: &Context<'_>) -> Poll<Result<(), Error>>
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
                &mut PollRequestBuf::new(&mut self.operation),
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

impl<'a, N, P: Access, A: AbstractBounceBufferAllocator, F: FnMut()> Future
    for RequestFuture<'a, N, P, A, F>
{
    type Output = Result<(), Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.poll_inner(cx)
    }
}

impl<'a, N, P: Access, A: AbstractBounceBufferAllocator, F: FnMut()> Drop
    for RequestFuture<'a, N, P, A, F>
{
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
