use alloc::rc::Rc;
use core::cell::RefCell;

use smoltcp::phy::{self, Device, DeviceCapabilities};
use smoltcp::time::Instant;
use smoltcp::wire::EthernetAddress;
use virtio_drivers::{device::net::*, transport::mmio::MmioTransport};

use crate::HalImpl;

const NET_QUEUE_SIZE: usize = 16;

type SharedVirtIONet = Rc<RefCell<VirtIONet<HalImpl, MmioTransport, NET_QUEUE_SIZE>>>;

pub(crate) struct DeviceImpl {
    shared_driver: SharedVirtIONet,
}

impl DeviceImpl {
    pub(crate) fn new(virtio_net: VirtIONet<HalImpl, MmioTransport, NET_QUEUE_SIZE>) -> Self {
        Self {
            shared_driver: Rc::new(RefCell::new(virtio_net)),
        }
    }

    fn shared_driver(&self) -> &SharedVirtIONet {
        &self.shared_driver
    }

    pub(crate) fn ack_interrupt(&self) {
        let _ = self.shared_driver().borrow_mut().ack_interrupt();
    }

    pub(crate) fn mac_address(&self) -> EthernetAddress {
        EthernetAddress(self.shared_driver().borrow().mac_address())
    }

    fn new_rx_token(&self, rx_buffer: RxBuffer) -> RxToken {
        RxToken {
            buffer: rx_buffer,
            shared_driver: self.shared_driver().clone(),
        }
    }

    fn new_tx_token(&self) -> TxToken {
        TxToken {
            shared_driver: self.shared_driver().clone(),
        }
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
            let rx_token = self.new_rx_token(driver.receive().unwrap());
            let tx_token = self.new_tx_token();
            Some((rx_token, tx_token))
        } else {
            None
        }
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(self.new_tx_token())
    }
}

pub(crate) struct RxToken {
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

pub(crate) struct TxToken {
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
