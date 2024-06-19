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

use sel4_driver_interfaces::net::GetNetDeviceMeta;
use sel4_driver_interfaces::HandleInterrupt;
use sel4_externally_shared::ExternallySharedRef;
use sel4_microkit::{Channel, Handler, Infallible, MessageInfo};
use sel4_microkit_message::MessageInfoExt as _;
use sel4_shared_ring_buffer::{roles::Use, RingBuffers};

use super::message_types::*;

pub struct HandlerImpl<Device> {
    dev: Device,
    client_region: ExternallySharedRef<'static, [u8]>,
    rx_ring_buffers: RingBuffers<'static, Use, fn()>,
    tx_ring_buffers: RingBuffers<'static, Use, fn()>,
    device_channel: Channel,
    client_channel: Channel,
}

impl<Device> HandlerImpl<Device> {
    pub fn new(
        dev: Device,
        client_region: ExternallySharedRef<'static, [u8]>,
        rx_ring_buffers: RingBuffers<'static, Use, fn()>,
        tx_ring_buffers: RingBuffers<'static, Use, fn()>,
        device_channel: Channel,
        client_channel: Channel,
    ) -> Self {
        Self {
            dev,
            client_region,
            rx_ring_buffers,
            tx_ring_buffers,
            device_channel,
            client_channel,
        }
    }
}

impl<Device: phy::Device + HandleInterrupt + GetNetDeviceMeta> Handler for HandlerImpl<Device> {
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
                        let start = desc.encoded_addr();
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
                self.rx_ring_buffers.notify();
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
                        let start = desc.encoded_addr();
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
                self.tx_ring_buffers.notify();
            }

            self.dev.handle_interrupt();
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
        if channel == self.client_channel {
            Ok(handle_client_request(&mut self.dev, msg_info))
        } else {
            unreachable!()
        }
    }
}

pub fn handle_client_request<T: GetNetDeviceMeta>(
    dev: &mut T,
    msg_info: MessageInfo,
) -> MessageInfo {
    match msg_info.recv_using_postcard::<Request>() {
        Ok(req) => {
            let resp: Response = match req {
                Request::GetMacAddress => dev
                    .get_mac_address()
                    .map(SuccessResponse::GetMacAddress)
                    .map_err(|_| ErrorResponse::Unspecified),
            };
            MessageInfo::send_using_postcard(resp).unwrap()
        }
        Err(_) => MessageInfo::send_unspecified_error(),
    }
}
