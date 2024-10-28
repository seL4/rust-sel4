//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2019-2020 rCore Developers
//
// SPDX-License-Identifier: MIT
//

#![no_std]

extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;
use core::convert::Infallible;

use log::trace;
use sel4_driver_interfaces::net::{GetNetDeviceMeta, MacAddress};
use sel4_driver_interfaces::HandleInterrupt;
use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::time::Instant;
use virtio_drivers::device::net::{RxBuffer, VirtIONet};
use virtio_drivers::{transport::Transport, Error, Hal};

pub const NET_QUEUE_SIZE: usize = 16;

pub type DeviceImpl<H, T> = VirtIONet<H, T, NET_QUEUE_SIZE>;

pub struct DeviceWrapper<H: Hal, T: Transport> {
    inner: Rc<RefCell<DeviceImpl<H, T>>>,
}

impl<H: Hal, T: Transport> DeviceWrapper<H, T> {
    pub fn new(dev: DeviceImpl<H, T>) -> Self {
        DeviceWrapper {
            inner: Rc::new(RefCell::new(dev)),
        }
    }
}

impl<H: Hal, T: Transport> HandleInterrupt for DeviceWrapper<H, T> {
    fn handle_interrupt(&mut self) {
        self.inner.borrow_mut().ack_interrupt();
    }
}

impl<H: Hal, T: Transport> GetNetDeviceMeta for DeviceWrapper<H, T> {
    type Error = Infallible;

    fn get_mac_address(&mut self) -> Result<MacAddress, Self::Error> {
        Ok(MacAddress(self.inner.borrow().mac_address()))
    }
}

impl<H: Hal, T: Transport> Device for DeviceWrapper<H, T> {
    type RxToken<'a>
        = VirtioRxToken<H, T>
    where
        Self: 'a;
    type TxToken<'a>
        = VirtioTxToken<H, T>
    where
        Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        match self.inner.borrow_mut().receive() {
            Ok(buf) => Some((
                VirtioRxToken(self.inner.clone(), buf),
                VirtioTxToken(self.inner.clone()),
            )),
            Err(Error::NotReady) => None,
            Err(err) => panic!("receive failed: {}", err),
        }
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(VirtioTxToken(self.inner.clone()))
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1536;
        caps.max_burst_size = Some(1);
        caps.medium = Medium::Ethernet;
        caps
    }
}

pub struct VirtioRxToken<H: Hal, T: Transport>(Rc<RefCell<DeviceImpl<H, T>>>, RxBuffer);

impl<H: Hal, T: Transport> RxToken for VirtioRxToken<H, T> {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut rx_buf = self.1;
        trace!(
            "RECV {} bytes: {:02X?}",
            rx_buf.packet_len(),
            rx_buf.packet()
        );
        let result = f(rx_buf.packet_mut());
        self.0.borrow_mut().recycle_rx_buffer(rx_buf).unwrap();
        result
    }
}

pub struct VirtioTxToken<H: Hal, T: Transport>(Rc<RefCell<DeviceImpl<H, T>>>);

impl<H: Hal, T: Transport> TxToken for VirtioTxToken<H, T> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut dev = self.0.borrow_mut();
        let mut tx_buf = dev.new_tx_buffer(len);
        let result = f(tx_buf.packet_mut());
        trace!("SEND {} bytes: {:02X?}", len, tx_buf.packet());
        dev.send(tx_buf).unwrap();
        result
    }
}
