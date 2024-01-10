//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use core::ptr::NonNull;

use virtio_drivers::{
    device::net::*,
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

use microkit_http_server_example_virtio_hal_impl::HalImpl;
use microkit_http_server_example_virtio_net_driver_interface_types::*;

mod config;

use config::channels;

const NET_QUEUE_SIZE: usize = 16;
const NET_BUFFER_LEN: usize = 2048;

#[protection_domain(
    heap_size = 512 * 1024,
)]
fn init() -> HandlerImpl {
    HalImpl::init(
        config::VIRTIO_NET_DRIVER_DMA_SIZE,
        *var!(virtio_net_driver_dma_vaddr: usize = 0),
        *var!(virtio_net_driver_dma_paddr: usize = 0),
    );

    let mut dev = {
        let header = NonNull::new(
            (*var!(virtio_net_mmio_vaddr: usize = 0) + config::VIRTIO_NET_MMIO_OFFSET)
                as *mut VirtIOHeader,
        )
        .unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        assert_eq!(transport.device_type(), DeviceType::Network);
        VirtIONet::<HalImpl, MmioTransport, NET_QUEUE_SIZE>::new(transport, NET_BUFFER_LEN).unwrap()
    };

    let client_region = unsafe {
        ExternallySharedRef::<'static, _>::new(
            memory_region_symbol!(virtio_net_client_dma_vaddr: *mut [u8], n = config::VIRTIO_NET_CLIENT_DMA_SIZE),
        )
    };

    let notify_client: fn() = || channels::CLIENT.notify();

    let rx_ring_buffers =
        RingBuffers::<'_, Use, fn()>::from_ptrs_using_default_initialization_strategy_for_role(
            unsafe { ExternallySharedRef::new(memory_region_symbol!(virtio_net_rx_free: *mut _)) },
            unsafe { ExternallySharedRef::new(memory_region_symbol!(virtio_net_rx_used: *mut _)) },
            notify_client,
        );

    let tx_ring_buffers =
        RingBuffers::<'_, Use, fn()>::from_ptrs_using_default_initialization_strategy_for_role(
            unsafe { ExternallySharedRef::new(memory_region_symbol!(virtio_net_tx_free: *mut _)) },
            unsafe { ExternallySharedRef::new(memory_region_symbol!(virtio_net_tx_used: *mut _)) },
            notify_client,
        );

    dev.ack_interrupt();
    channels::DEVICE.irq_ack().unwrap();

    HandlerImpl {
        dev,
        client_region,
        rx_ring_buffers,
        tx_ring_buffers,
    }
}

struct HandlerImpl {
    dev: VirtIONet<HalImpl, MmioTransport, NET_QUEUE_SIZE>,
    client_region: ExternallySharedRef<'static, [u8]>,
    rx_ring_buffers: RingBuffers<'static, Use, fn()>,
    tx_ring_buffers: RingBuffers<'static, Use, fn()>,
}

impl Handler for HandlerImpl {
    type Error = Infallible;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        match channel {
            channels::DEVICE | channels::CLIENT => {
                let mut notify_rx = false;

                while self.dev.can_recv() && !self.rx_ring_buffers.free_mut().is_empty().unwrap() {
                    let rx_buf = self.dev.receive().unwrap();
                    let desc = self.rx_ring_buffers.free_mut().dequeue().unwrap().unwrap();
                    let desc_len = usize::try_from(desc.len()).unwrap();
                    assert!(desc_len >= rx_buf.packet_len());
                    let buf_range = {
                        let start = desc.encoded_addr();
                        start..start + rx_buf.packet_len()
                    };
                    self.client_region
                        .as_mut_ptr()
                        .index(buf_range)
                        .copy_from_slice(rx_buf.packet());
                    self.dev.recycle_rx_buffer(rx_buf).unwrap();
                    self.rx_ring_buffers
                        .used_mut()
                        .enqueue_and_commit(desc)
                        .unwrap()
                        .unwrap();
                    notify_rx = true;
                }

                if notify_rx {
                    self.rx_ring_buffers.notify();
                }

                let mut notify_tx = false;

                while !self.tx_ring_buffers.free_mut().is_empty().unwrap() && self.dev.can_send() {
                    let desc = self.tx_ring_buffers.free_mut().dequeue().unwrap().unwrap();
                    let buf_range = {
                        let start = desc.encoded_addr();
                        start..start + usize::try_from(desc.len()).unwrap()
                    };
                    let mut tx_buf = self.dev.new_tx_buffer(buf_range.len());
                    self.client_region
                        .as_ptr()
                        .index(buf_range)
                        .copy_into_slice(tx_buf.packet_mut());
                    self.dev.send(tx_buf).unwrap();
                    self.tx_ring_buffers
                        .used_mut()
                        .enqueue_and_commit(desc)
                        .unwrap()
                        .unwrap();
                    notify_tx = true;
                }

                if notify_tx {
                    self.tx_ring_buffers.notify();
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
                    Request::GetMacAddress => {
                        let mac_address = self.dev.mac_address();
                        MessageInfo::send_using_postcard(GetMacAddressResponse {
                            mac_address: MacAddress(mac_address),
                        })
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
