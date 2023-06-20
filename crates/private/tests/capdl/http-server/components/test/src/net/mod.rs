use alloc::rc::Rc;
use core::cell::RefCell;

use smoltcp::phy::{self, Device, DeviceCapabilities};
use smoltcp::time::Instant;
use smoltcp::wire::EthernetAddress;
use virtio_drivers::{device::net::*, transport::mmio::MmioTransport};

mod hal;

pub use hal::HalImpl;

const NET_QUEUE_SIZE: usize = 16;

pub type SharedVirtIONet = Rc<RefCell<VirtIONet<HalImpl, MmioTransport, NET_QUEUE_SIZE>>>;

pub struct Net {
    device: SharedVirtIONet,
}

impl Net {
    pub fn new(virtio_net: VirtIONet<HalImpl, MmioTransport, NET_QUEUE_SIZE>) -> Self {
        Net {
            device: Rc::new(RefCell::new(virtio_net)),
        }
    }

    pub fn device(&self) -> &SharedVirtIONet {
        &self.device
    }

    pub fn mac_address(&self) -> EthernetAddress {
        EthernetAddress(self.device().borrow().mac_address())
    }
}

impl Device for Net {
    type RxToken<'a> = RxToken;
    type TxToken<'a> = TxToken;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut cap = DeviceCapabilities::default();
        cap.max_transmission_unit = 1500;
        cap
    }

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let mut device = self.device.borrow_mut();
        if device.can_recv() {
            let rx_buffer = device.receive().unwrap();
            let rx_token = RxToken {
                buffer: rx_buffer,
                device: self.device.clone(),
            };
            let tx_token = TxToken {
                device: self.device.clone(),
            };
            Some((rx_token, tx_token))
        } else {
            None
        }
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(TxToken {
            device: self.device.clone(),
        })
    }
}

pub struct RxToken {
    buffer: RxBuffer,
    device: SharedVirtIONet,
}

impl phy::RxToken for RxToken {
    fn consume<R, F>(mut self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let r = f(self.buffer.packet_mut());
        self.device
            .borrow_mut()
            .recycle_rx_buffer(self.buffer)
            .unwrap();
        r
    }
}

pub struct TxToken {
    device: SharedVirtIONet,
}

impl phy::TxToken for TxToken {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut tx_buffer = self.device.borrow().new_tx_buffer(len);
        let r = f(tx_buffer.packet_mut());
        self.device.borrow_mut().send(tx_buffer).unwrap();
        r
    }
}
