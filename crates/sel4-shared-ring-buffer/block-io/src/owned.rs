//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::alloc::Layout;
use core::task::{Poll, Waker};

use sel4_async_block_io::{access::Access, Operation};
use sel4_bounce_buffer_allocator::{AbstractBounceBufferAllocator, BounceBufferAllocator};
use sel4_externally_shared::ExternallySharedRef;
use sel4_shared_ring_buffer::{
    roles::Provide, Descriptor, PeerMisbehaviorError as SharedRingBuffersPeerMisbehaviorError,
    RingBuffers,
};
use sel4_shared_ring_buffer_block_io_types::{
    BlockIORequest, BlockIORequestStatus, BlockIORequestType,
};
use sel4_shared_ring_buffer_bookkeeping::{slot_set_semaphore::*, slot_tracker::*};

pub use crate::errors::{Error, ErrorOrUserError, IOError, PeerMisbehaviorError, UserError};

pub struct OwnedSharedRingBufferBlockIO<S, A, F> {
    dma_region: ExternallySharedRef<'static, [u8]>,
    bounce_buffer_allocator: BounceBufferAllocator<A>,
    ring_buffers: RingBuffers<'static, Provide, F, BlockIORequest>,
    requests: SlotTracker<StateTypesImpl>,
    slot_set_semaphore: SlotSetSemaphore<S, NUM_SLOT_POOLS>,
}

const RING_BUFFERS_SLOT_POOL_INDEX: usize = 0;
const REQUESTS_SLOT_POOL_INDEX: usize = 1;
const NUM_SLOT_POOLS: usize = 2;

enum StateTypesImpl {}

impl SlotStateTypes for StateTypesImpl {
    type Common = ();
    type Free = ();
    type Occupied = Occupied;
}

struct Occupied {
    req: BlockIORequest,
    state: OccupiedState,
}

enum OccupiedState {
    Pending { waker: Option<Waker> },
    Canceled,
    Complete { error: Option<IOError> },
}

pub enum IssueRequestBuf<'a> {
    Read { len: usize },
    Write { buf: &'a [u8] },
}

impl<'a> IssueRequestBuf<'a> {
    pub fn new<A: Access>(operation: &'a Operation<'a, A>) -> Self {
        match operation {
            Operation::Read { buf, .. } => Self::Read { len: buf.len() },
            Operation::Write { buf, .. } => Self::Write { buf },
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Read { len } => *len,
            Self::Write { buf } => buf.len(),
        }
    }

    fn ty(&self) -> BlockIORequestType {
        match self {
            Self::Read { .. } => BlockIORequestType::Read,
            Self::Write { .. } => BlockIORequestType::Write,
        }
    }
}

pub enum PollRequestBuf<'a> {
    Read { buf: &'a mut [u8] },
    Write,
}

impl<'a> PollRequestBuf<'a> {
    pub fn new<'b, A: Access>(operation: &'a mut Operation<'b, A>) -> Self
    where
        'b: 'a,
    {
        match operation {
            Operation::Read { buf, .. } => Self::Read { buf },
            Operation::Write { .. } => Self::Write,
        }
    }
}

impl<S: SlotSemaphore, A: AbstractBounceBufferAllocator, F: FnMut()>
    OwnedSharedRingBufferBlockIO<S, A, F>
{
    pub fn new(
        dma_region: ExternallySharedRef<'static, [u8]>,
        bounce_buffer_allocator: BounceBufferAllocator<A>,
        mut ring_buffers: RingBuffers<'static, Provide, F, BlockIORequest>,
    ) -> Self {
        assert!(ring_buffers.free_mut().is_empty().unwrap());
        assert!(ring_buffers.used_mut().is_empty().unwrap());
        let n = ring_buffers.free().capacity();
        Self {
            dma_region,
            bounce_buffer_allocator,
            ring_buffers,
            requests: SlotTracker::new_with_capacity((), (), n),
            slot_set_semaphore: SlotSetSemaphore::new([n, n]),
        }
    }

    pub fn slot_set_semaphore(&self) -> &SlotSetSemaphoreHandle<S, NUM_SLOT_POOLS> {
        self.slot_set_semaphore.handle()
    }

    fn report_current_num_free_current_num_free_ring_buffers_slots(
        &mut self,
    ) -> Result<(), ErrorOrUserError> {
        let current_num_free = self.requests.num_free();
        self.slot_set_semaphore
            .report_current_num_free_slots(RING_BUFFERS_SLOT_POOL_INDEX, current_num_free)
            .unwrap();
        Ok(())
    }

    fn report_current_num_free_current_num_free_requests_slots(
        &mut self,
    ) -> Result<(), ErrorOrUserError> {
        let current_num_free = self.ring_buffers.free_mut().num_empty_slots()?;
        self.slot_set_semaphore
            .report_current_num_free_slots(REQUESTS_SLOT_POOL_INDEX, current_num_free)
            .unwrap();
        Ok(())
    }

    fn can_issue_requests(
        &mut self,
        n: usize,
    ) -> Result<bool, SharedRingBuffersPeerMisbehaviorError> {
        let can =
            self.ring_buffers.free_mut().num_empty_slots()? >= n && self.requests.num_free() >= n;
        Ok(can)
    }

    pub fn issue_read_request(
        &mut self,
        reservation: &mut SlotSetReservation<'_, S, NUM_SLOT_POOLS>,
        start_block_idx: u64,
        num_bytes: usize,
    ) -> Result<usize, ErrorOrUserError> {
        self.issue_request(
            reservation,
            start_block_idx,
            &mut IssueRequestBuf::Read { len: num_bytes },
        )
    }

    pub fn issue_write_request(
        &mut self,
        reservation: &mut SlotSetReservation<'_, S, NUM_SLOT_POOLS>,
        start_block_idx: u64,
        buf: &[u8],
    ) -> Result<usize, ErrorOrUserError> {
        self.issue_request(
            reservation,
            start_block_idx,
            &mut IssueRequestBuf::Write { buf },
        )
    }

    pub fn issue_request(
        &mut self,
        reservation: &mut SlotSetReservation<'_, S, NUM_SLOT_POOLS>,
        start_block_idx: u64,
        buf: &mut IssueRequestBuf,
    ) -> Result<usize, ErrorOrUserError> {
        if reservation.count() < 1 {
            return Err(UserError::TooManyOutstandingRequests.into());
        }

        assert!(self.can_issue_requests(1)?);

        let request_index = self.requests.peek_next_free_index().unwrap();

        let range = self
            .bounce_buffer_allocator
            .allocate(Layout::from_size_align(buf.len(), 1).unwrap())
            .map_err(|_| Error::BounceBufferAllocationError)?;

        if let IssueRequestBuf::Write { buf } = buf {
            self.dma_region
                .as_mut_ptr()
                .index(range.clone())
                .copy_from_slice(buf);
        }

        let req = BlockIORequest::new(
            BlockIORequestStatus::Pending,
            buf.ty(),
            start_block_idx.try_into().unwrap(),
            Descriptor::from_encoded_addr_range(range, request_index),
        );

        self.requests
            .occupy(Occupied {
                req,
                state: OccupiedState::Pending { waker: None },
            })
            .unwrap();

        self.ring_buffers
            .free_mut()
            .enqueue_and_commit(req)?
            .unwrap();

        self.ring_buffers.notify_mut();

        self.slot_set_semaphore.consume(reservation, 1).unwrap();

        Ok(request_index)
    }

    pub fn cancel_request(&mut self, request_index: usize) -> Result<(), ErrorOrUserError> {
        let state_value = self.requests.get_state_value_mut(request_index)?;
        let occupied = state_value.as_occupied()?;
        match &occupied.state {
            OccupiedState::Pending { .. } => {
                occupied.state = OccupiedState::Canceled;
            }
            OccupiedState::Complete { .. } => {
                let range = occupied.req.buf().encoded_addr_range();
                self.bounce_buffer_allocator.deallocate(range);
                self.requests.free(request_index, ()).unwrap();
                self.report_current_num_free_current_num_free_requests_slots()?;
            }
            _ => {
                return Err(UserError::RequestStateMismatch.into());
            }
        }
        Ok(())
    }

    pub fn poll_read_request(
        &mut self,
        request_index: usize,
        buf: &mut [u8],
        waker: Option<Waker>,
    ) -> Result<Poll<Result<(), IOError>>, ErrorOrUserError> {
        self.poll_request(request_index, &mut PollRequestBuf::Read { buf }, waker)
    }

    pub fn poll_write_request(
        &mut self,
        request_index: usize,
        waker: Option<Waker>,
    ) -> Result<Poll<Result<(), IOError>>, ErrorOrUserError> {
        self.poll_request(request_index, &mut PollRequestBuf::Write, waker)
    }

    pub fn poll_request(
        &mut self,
        request_index: usize,
        buf: &mut PollRequestBuf,
        waker: Option<Waker>,
    ) -> Result<Poll<Result<(), IOError>>, ErrorOrUserError> {
        let state_value = self.requests.get_state_value_mut(request_index)?;
        let occupied = state_value.as_occupied()?;

        Ok(match &mut occupied.state {
            OccupiedState::Pending {
                waker: ref mut waker_slot,
            } => {
                if let Some(waker) = waker {
                    waker_slot.replace(waker);
                }
                Poll::Pending
            }
            OccupiedState::Complete { error } => {
                let val = match error {
                    None => Ok(()),
                    Some(err) => Err(err.clone()),
                };

                let range = occupied.req.buf().encoded_addr_range();

                match buf {
                    PollRequestBuf::Read { buf } => {
                        self.dma_region
                            .as_mut_ptr()
                            .index(range.clone())
                            .copy_into_slice(buf);
                    }
                    PollRequestBuf::Write => {}
                }

                self.bounce_buffer_allocator.deallocate(range);

                self.requests.free(request_index, ()).unwrap();
                self.report_current_num_free_current_num_free_requests_slots()?;

                Poll::Ready(val)
            }
            _ => {
                return Err(UserError::RequestStateMismatch.into());
            }
        })
    }

    pub fn poll(&mut self) -> Result<bool, ErrorOrUserError> {
        self.report_current_num_free_current_num_free_ring_buffers_slots()?;

        let mut notify = false;

        while let Some(completed_req) = self.ring_buffers.used_mut().dequeue()? {
            let request_index = completed_req.buf().cookie();

            let state_value = self
                .requests
                .get_state_value_mut(request_index)
                .map_err(|_| PeerMisbehaviorError::OutOfBoundsCookie)?;

            let occupied = state_value
                .as_occupied()
                .map_err(|_| PeerMisbehaviorError::StateMismatch)?;

            {
                let mut observed_request = completed_req;
                observed_request.set_status(BlockIORequestStatus::Pending);
                if observed_request != occupied.req {
                    return Err(PeerMisbehaviorError::DescriptorMismatch.into());
                }
            }

            match &mut occupied.state {
                OccupiedState::Pending { waker } => {
                    let waker = waker.take();

                    let status = completed_req
                        .status()
                        .map_err(|_| PeerMisbehaviorError::InvalidDescriptor)?;

                    occupied.state = OccupiedState::Complete {
                        error: match status {
                            BlockIORequestStatus::Pending => {
                                return Err(PeerMisbehaviorError::InvalidDescriptor.into());
                            }
                            BlockIORequestStatus::Ok => None,
                            BlockIORequestStatus::IOError => Some(IOError),
                        },
                    };

                    if let Some(waker) = waker {
                        waker.wake();
                    }
                }
                OccupiedState::Canceled => {
                    let range = occupied.req.buf().encoded_addr_range();
                    self.bounce_buffer_allocator.deallocate(range);
                    self.requests.free(request_index, ()).unwrap();
                    self.report_current_num_free_current_num_free_requests_slots()?;
                }
                _ => {
                    return Err(UserError::RequestStateMismatch.into());
                }
            }

            notify = true;
        }

        if notify {
            self.ring_buffers.notify_mut();
        }

        Ok(notify)
    }
}
