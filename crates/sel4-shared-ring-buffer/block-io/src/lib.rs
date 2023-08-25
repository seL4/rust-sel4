#![no_std]
#![feature(async_fn_in_trait)]
#![feature(never_type)]
#![feature(strict_provenance)]

extern crate alloc;

use alloc::rc::Rc;
use core::alloc::Layout;
use core::cell::RefCell;
use core::task::{ready, Poll};

use async_unsync::semaphore::Semaphore;
use futures::prelude::*;

use sel4_async_block_io::BlockIO as BlockIOTrait;
use sel4_async_request_statuses::RequestStatuses;
use sel4_bounce_buffer_allocator::{Basic, BounceBufferAllocator};
use sel4_externally_shared::ExternallySharedRef;
use sel4_shared_ring_buffer::{
    Descriptor, Error as SharedRingBuffersError, RingBuffers, RING_BUFFER_SIZE,
};
use sel4_shared_ring_buffer_block_io_types::{
    BlockIORequest, BlockIORequestStatus, BlockIORequestType,
};

pub const BLOCK_SIZE: usize = 512;

#[derive(Clone)]
pub struct BlockIO {
    shared_inner: Rc<RefCell<Inner>>,
}

type EncodedAddr = usize;

struct Inner {
    dma_region: ExternallySharedRef<'static, [u8]>,
    dma_region_paddr: usize,
    bounce_buffer_allocator: BounceBufferAllocator<Basic>,
    ring_buffers: RingBuffers<'static, fn() -> Result<(), !>, BlockIORequest>,
    request_statuses: RequestStatuses<EncodedAddr, BlockIORequest, BlockIORequestStatus>,
    queue_guard: Rc<Semaphore>,
}

impl BlockIO {
    pub fn new(
        dma_region: ExternallySharedRef<'static, [u8]>,
        dma_region_paddr: usize,
        ring_buffers: RingBuffers<'static, fn() -> Result<(), !>, BlockIORequest>,
    ) -> Self {
        let max_alignment = 1
            << dma_region
                .as_ptr()
                .as_raw_ptr()
                .addr()
                .trailing_zeros()
                .min(dma_region_paddr.trailing_zeros());

        let bounce_buffer_allocator =
            BounceBufferAllocator::new(Basic::new(dma_region.as_ptr().len()), max_alignment);

        Self {
            shared_inner: Rc::new(RefCell::new(Inner {
                dma_region,
                dma_region_paddr,
                bounce_buffer_allocator,
                ring_buffers,
                request_statuses: RequestStatuses::new(),
                queue_guard: Rc::new(Semaphore::new(RING_BUFFER_SIZE)),
            })),
        }
    }

    pub fn poll(&self) -> bool {
        let mut inner = self.shared_inner.borrow_mut();

        let mut notify = false;

        while let Ok(mut completed_req) = inner
            .ring_buffers
            .used_mut()
            .dequeue()
            .map_err(|err| assert_eq!(err, SharedRingBuffersError::RingIsEmpty))
        {
            let status = completed_req.status().unwrap();
            let key = completed_req.buf().encoded_addr() - inner.dma_region_paddr;
            completed_req.set_status(BlockIORequestStatus::Pending);
            let expected_req = completed_req;
            let actual_req = inner.request_statuses.get(&key).unwrap();
            assert_eq!(&expected_req, actual_req);
            inner.request_statuses.mark_complete(&key, status).unwrap();
            notify = true;
        }

        if notify {
            inner.ring_buffers.notify().unwrap();
        }

        notify
    }
}

impl BlockIOTrait<BLOCK_SIZE> for BlockIO {
    async fn read_block(&self, block_id: usize, buf: &mut [u8; BLOCK_SIZE]) {
        let sem = self.shared_inner.borrow().queue_guard.clone();
        let permit = sem.acquire().await;

        let key = {
            let mut inner = self.shared_inner.borrow_mut();
            let range = inner
                .bounce_buffer_allocator
                .allocate(Layout::from_size_align(buf.len(), 1).unwrap())
                .unwrap();
            let key = range.start;
            let req = BlockIORequest::new(
                BlockIORequestStatus::Pending,
                BlockIORequestType::Read,
                block_id,
                Descriptor::new(
                    inner.dma_region_paddr + range.start,
                    range.len().try_into().unwrap(),
                    0,
                ),
            );
            inner.request_statuses.add(key, req).unwrap();
            inner.ring_buffers.free_mut().enqueue(req).unwrap();
            inner.ring_buffers.notify().unwrap();
            key
        };

        future::poll_fn(|cx| {
            let mut inner = self.shared_inner.borrow_mut();
            let completion = ready!(inner.request_statuses.poll(&key, cx.waker()).unwrap());
            assert_eq!(completion.complete, BlockIORequestStatus::Ok);
            let req = completion.value;
            let range_start = req.buf().encoded_addr() - inner.dma_region_paddr;
            let range_end = range_start + usize::try_from(req.buf().len()).unwrap();
            let range = range_start..range_end;
            inner
                .dma_region
                .as_mut_ptr()
                .index(range.clone())
                .copy_into_slice(buf);
            inner.bounce_buffer_allocator.deallocate(range);
            Poll::Ready(())
        })
        .await;

        drop(permit); // explicit extent of scope
    }
}
