//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Galois, Inc.
//
// SPDX-License-Identifier: BSD-2-Clause
//

//! A generic microkit handler for implementors of [`smoltcp::phy::Device`].

use smoltcp::{
    phy::{self, RxToken, TxToken},
    time::Instant,
};

use sel4_externally_shared::ExternallySharedRef;
use sel4_microkit::{Channel, Handler, Infallible, MessageInfo};
use sel4_microkit_message::MessageInfoExt as _;
use sel4_shared_ring_buffer::{roles::Use, RingBuffers};

use super::common::*;

pub trait IrqAck {
    fn irq_ack(&mut self);
}

pub trait HasMac {
    fn mac_address(&self) -> [u8; 6];
}

pub struct Driver<Device> {
    dev: Device,
    client_region: ExternallySharedRef<'static, [u8]>,
    client_region_paddr: usize,
    rx_ring_buffers: RingBuffers<'static, Use, fn() -> Result<(), Infallible>>,
    tx_ring_buffers: RingBuffers<'static, Use, fn() -> Result<(), Infallible>>,
    device_channel: Channel,
    client_channel: Channel,
}

impl<Device> Driver<Device> {
    pub fn new(
        dev: Device,
        client_region: ExternallySharedRef<'static, [u8]>,
        client_region_paddr: usize,
        rx_ring_buffers: RingBuffers<'static, Use, fn() -> Result<(), Infallible>>,
        tx_ring_buffers: RingBuffers<'static, Use, fn() -> Result<(), Infallible>>,
        device_channel: Channel,
        client_channel: Channel,
    ) -> Self {
        // XXX We could maybe initialize DMA here, so we don't need to do
        // it in main. Also maybe initialize the ring buffers.
        Self {
            dev,
            client_region,
            client_region_paddr,
            rx_ring_buffers,
            tx_ring_buffers,
            device_channel,
            client_channel,
        }
    }
}

impl<Device: phy::Device + IrqAck + HasMac> Handler for Driver<Device> {
    type Error = Infallible;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        if channel == self.device_channel || channel == self.client_channel {
            let mut notify_rx = false;

            while !self.rx_ring_buffers.free_mut().is_empty().unwrap() {
                let rx_tok = match self.dev.receive(Instant::ZERO) {
                    Some((rx_tok, _tx_tok)) => rx_tok,
                    None => break,
                };

                let desc = self.rx_ring_buffers.free_mut().dequeue().unwrap().unwrap();
                let desc_len = usize::try_from(desc.len()).unwrap();

                rx_tok.consume(|rx_buf| {
                    assert!(desc_len >= rx_buf.len());
                    let buf_range = {
                        let start = desc.encoded_addr() - self.client_region_paddr;
                        start..start + rx_buf.len()
                    };
                    self.client_region
                        .as_mut_ptr()
                        .index(buf_range)
                        .copy_from_slice(&rx_buf);
                });

                self.rx_ring_buffers
                    .used_mut()
                    .enqueue(desc, true)
                    .unwrap()
                    .unwrap();
                notify_rx = true;
            }

            if notify_rx {
                self.rx_ring_buffers.notify().unwrap();
            }

            let mut notify_tx = false;

            while !self.tx_ring_buffers.free_mut().is_empty().unwrap() {
                let tx_tok = match self.dev.transmit(Instant::ZERO) {
                    Some(tx_tok) => tx_tok,
                    None => break,
                };

                let desc = self.tx_ring_buffers.free_mut().dequeue().unwrap().unwrap();
                let tx_len = usize::try_from(desc.len()).unwrap();

                tx_tok.consume(tx_len, |tx_buf| {
                    let buf_range = {
                        let start = desc.encoded_addr() - self.client_region_paddr;
                        start..start + tx_len
                    };
                    self.client_region
                        .as_ptr()
                        .index(buf_range)
                        .copy_into_slice(tx_buf);
                });

                self.tx_ring_buffers
                    .used_mut()
                    .enqueue(desc, true)
                    .unwrap()
                    .unwrap();
                notify_tx = true;
            }

            if notify_tx {
                self.tx_ring_buffers.notify().unwrap();
            }

            self.dev.irq_ack();
            self.device_channel.irq_ack().unwrap();
        } else {
            unreachable!()
        }

        Ok(())
    }

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        Ok(if channel == self.client_channel {
            match msg_info.recv_using_postcard::<Request>() {
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
            }
        } else {
            unreachable!()
        })
    }
}
