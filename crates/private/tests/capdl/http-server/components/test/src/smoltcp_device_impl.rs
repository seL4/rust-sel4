use alloc::rc::Rc;
use core::cell::RefCell;

use smoltcp::phy::{self, Device, DeviceCapabilities};
use smoltcp::time::Instant;
use smoltcp::wire::EthernetAddress;
use virtio_drivers::{device::net::*, transport::mmio::MmioTransport};

use crate::HalImpl;

const NET_QUEUE_SIZE: usize = 16;

pub type SharedVirtIONet = Rc<RefCell<VirtIONet<HalImpl, MmioTransport, NET_QUEUE_SIZE>>>;

pub struct DeviceImpl {
    shared_driver: SharedVirtIONet,
}

impl DeviceImpl {
    pub fn new(virtio_net: VirtIONet<HalImpl, MmioTransport, NET_QUEUE_SIZE>) -> Self {
        Self {
            shared_driver: Rc::new(RefCell::new(virtio_net)),
        }
    }

    fn shared_driver(&self) -> &SharedVirtIONet {
        &self.shared_driver
    }

    pub fn ack_interrupt(&self) {
        let _ = self.shared_driver().borrow_mut().ack_interrupt();
    }

    pub fn mac_address(&self) -> EthernetAddress {
        EthernetAddress(self.shared_driver().borrow().mac_address())
    }
}

impl Device for DeviceImpl {
    type RxToken<'a> = RxToken;
    type TxToken<'a> = TxToken;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut cap = DeviceCapabilities::default();
        cap.max_transmission_unit = 1500;
        cap
    }

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let mut driver = self.shared_driver().borrow_mut();
        if driver.can_recv() {
            let rx_buffer = driver.receive().unwrap();
            let rx_token = RxToken {
                buffer: rx_buffer,
                shared_driver: self.shared_driver().clone(),
            };
            let tx_token = TxToken {
                shared_driver: self.shared_driver().clone(),
            };
            Some((rx_token, tx_token))
        } else {
            None
        }
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(TxToken {
            shared_driver: self.shared_driver().clone(),
        })
    }
}

pub struct RxToken {
    buffer: RxBuffer,
    shared_driver: SharedVirtIONet,
}

impl phy::RxToken for RxToken {
    fn consume<R, F>(mut self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let r = f(self.buffer.packet_mut());
        self.shared_driver
            .borrow_mut()
            .recycle_rx_buffer(self.buffer)
            .unwrap();
        r
    }
}

pub struct TxToken {
    shared_driver: SharedVirtIONet,
}

impl phy::TxToken for TxToken {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut tx_buffer = self.shared_driver.borrow().new_tx_buffer(len);
        let r = f(tx_buffer.packet_mut());
        self.shared_driver.borrow_mut().send(tx_buffer).unwrap();
        r
    }
}
