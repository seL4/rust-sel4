//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::pin::Pin;
use core::ptr::NonNull;

use virtio_drivers::{
    device::blk::*,
    transport::{
        mmio::{MmioTransport, VirtIOHeader},
        DeviceType, Transport,
    },
};

use sel4_externally_shared::{ExternallySharedRef, ExternallySharedRefExt};
use sel4_microkit::{
    memory_region_symbol, protection_domain, var, Channel, Handler, Infallible, MessageInfo,
};
use sel4_microkit_message::MessageInfoExt as _;
use sel4_shared_ring_buffer::{roles::Use, RingBuffers};
use sel4_shared_ring_buffer_block_io_types::{
    BlockIORequest, BlockIORequestStatus, BlockIORequestType,
};

use microkit_http_server_example_virtio_blk_driver_interface_types::*;
use microkit_http_server_example_virtio_hal_impl::HalImpl;

mod config;

use config::channels;

// HACK hard-coded in virtio-drivers
const QUEUE_SIZE: usize = 4;

#[protection_domain(
    heap_size = 64 * 1024,
)]
fn init() -> HandlerImpl {
    HalImpl::init(
        config::VIRTIO_BLK_DRIVER_DMA_SIZE,
        *var!(virtio_blk_driver_dma_vaddr: usize = 0),
        *var!(virtio_blk_driver_dma_paddr: usize = 0),
    );

    let mut dev = {
        let header = NonNull::new(
            (*var!(virtio_blk_mmio_vaddr: usize = 0) + config::VIRTIO_BLK_MMIO_OFFSET)
                as *mut VirtIOHeader,
        )
        .unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        assert_eq!(transport.device_type(), DeviceType::Block);
        VirtIOBlk::<HalImpl, MmioTransport>::new(transport).unwrap()
    };

    let client_region = unsafe {
        ExternallySharedRef::<'static, _>::new(
            memory_region_symbol!(virtio_blk_client_dma_vaddr: *mut [u8], n = config::VIRTIO_BLK_CLIENT_DMA_SIZE),
        )
    };

    let notify_client: fn() = || channels::CLIENT.notify();

    let ring_buffers =
        RingBuffers::<'_, Use, fn(), BlockIORequest>::from_ptrs_using_default_initialization_strategy_for_role(
            unsafe { ExternallySharedRef::new(memory_region_symbol!(virtio_blk_free: *mut _)) },
            unsafe { ExternallySharedRef::new(memory_region_symbol!(virtio_blk_used: *mut _)) },
            notify_client,
        );

    dev.ack_interrupt();
    channels::DEVICE.irq_ack().unwrap();

    HandlerImpl {
        dev,
        client_region,
        ring_buffers,
        pending: BTreeMap::new(),
    }
}

struct HandlerImpl {
    dev: VirtIOBlk<HalImpl, MmioTransport>,
    client_region: ExternallySharedRef<'static, [u8]>,
    ring_buffers: RingBuffers<'static, Use, fn(), BlockIORequest>,
    pending: BTreeMap<u16, Pin<Box<PendingEntry>>>,
}

struct PendingEntry {
    client_req: BlockIORequest,
    virtio_req: BlkReq,
    virtio_resp: BlkResp,
}

impl Handler for HandlerImpl {
    type Error = Infallible;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        match channel {
            channels::DEVICE | channels::CLIENT => {
                let mut notify = false;

                while self.dev.peek_used().is_some() {
                    let token = self.dev.peek_used().unwrap();
                    let mut pending_entry = self.pending.remove(&token).unwrap();
                    let buf_range = {
                        let start = pending_entry.client_req.buf().encoded_addr();
                        let len = usize::try_from(pending_entry.client_req.buf().len()).unwrap();
                        start..start + len
                    };
                    let mut buf_ptr = self
                        .client_region
                        .as_mut_ptr()
                        .index(buf_range)
                        .as_raw_ptr();
                    unsafe {
                        let pending_entry = &mut *pending_entry;
                        self.dev
                            .complete_read_block(
                                token,
                                &pending_entry.virtio_req,
                                buf_ptr.as_mut(),
                                &mut pending_entry.virtio_resp,
                            )
                            .unwrap();
                    }
                    let status = match pending_entry.virtio_resp.status() {
                        RespStatus::OK => BlockIORequestStatus::Ok,
                        _ => panic!(),
                    };
                    let mut completed_req = pending_entry.client_req;
                    completed_req.set_status(status);
                    self.ring_buffers
                        .used_mut()
                        .enqueue_and_commit(completed_req)
                        .unwrap()
                        .unwrap();
                    notify = true;
                }

                while self.pending.len() < QUEUE_SIZE
                    && !self.ring_buffers.free_mut().is_empty().unwrap()
                {
                    let client_req = self.ring_buffers.free_mut().dequeue().unwrap().unwrap();
                    assert_eq!(client_req.ty().unwrap(), BlockIORequestType::Read);
                    let mut pending_entry = Box::pin(PendingEntry {
                        client_req,
                        virtio_req: BlkReq::default(),
                        virtio_resp: BlkResp::default(),
                    });
                    let buf_range = {
                        let start = client_req.buf().encoded_addr();
                        let len = usize::try_from(client_req.buf().len()).unwrap();
                        start..start + len
                    };
                    let mut buf_ptr = self
                        .client_region
                        .as_mut_ptr()
                        .index(buf_range)
                        .as_raw_ptr();
                    assert_eq!(buf_ptr.len(), 512);
                    let token = unsafe {
                        let pending_entry = &mut *pending_entry;
                        self.dev
                            .read_block_nb(
                                pending_entry
                                    .client_req
                                    .start_block_idx()
                                    .try_into()
                                    .unwrap(),
                                &mut pending_entry.virtio_req,
                                buf_ptr.as_mut(),
                                &mut pending_entry.virtio_resp,
                            )
                            .unwrap()
                    };
                    assert!(self.pending.insert(token, pending_entry).is_none());
                    notify = true;
                }

                if notify {
                    self.ring_buffers.notify();
                }

                self.dev.ack_interrupt();
                channels::DEVICE.irq_ack().unwrap();
            }
            _ => {
                unreachable!()
            }
        }
        Ok(())
    }

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        Ok(match channel {
            channels::CLIENT => match msg_info.recv_using_postcard::<Request>() {
                Ok(req) => match req {
                    Request::GetNumBlocks => {
                        let num_blocks = self.dev.capacity();
                        MessageInfo::send_using_postcard(GetNumBlocksResponse { num_blocks })
                            .unwrap()
                    }
                },
                Err(_) => MessageInfo::send_unspecified_error(),
            },
            _ => {
                unreachable!()
            }
        })
    }
}
