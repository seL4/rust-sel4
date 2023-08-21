#![no_std]
#![feature(never_type)]
#![feature(strict_provenance)]

extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;

use smoltcp::phy::{self, Device, DeviceCapabilities};
use smoltcp::time::Instant;

use sel4_externally_shared::ExternallySharedRef;
use sel4_shared_ring_buffer::RingBuffers;

mod inner;

use inner::{Inner, RxBufferIndex, TxBufferIndex};

pub struct DeviceImpl {
    shared_inner: SharedInner,
}

type SharedInner = Rc<RefCell<Inner>>;

impl DeviceImpl {
    pub fn new(
        dma_region: ExternallySharedRef<'static, [u8]>,
        dma_region_paddr: usize,
        rx_ring_buffers: RingBuffers<'static, fn() -> Result<(), !>>,
        tx_ring_buffers: RingBuffers<'static, fn() -> Result<(), !>>,
        num_rx_buffers: usize,
        rx_buffer_size: usize,
        mtu: usize,
    ) -> Self {
        Self {
            shared_inner: Rc::new(RefCell::new(Inner::new(
                dma_region,
                dma_region_paddr,
                rx_ring_buffers,
                tx_ring_buffers,
                num_rx_buffers,
                rx_buffer_size,
                mtu,
            ))),
        }
    }

    fn shared_inner(&self) -> &SharedInner {
        &self.shared_inner
    }

    pub fn handle_notification(&self) {
        self.shared_inner().borrow_mut().handle_notification()
    }

    fn new_rx_token(&self, rx_buffer: RxBufferIndex) -> RxToken {
        RxToken {
            buffer: rx_buffer,
            shared_inner: self.shared_inner().clone(),
        }
    }

    fn new_tx_token(&self, tx_buffer: TxBufferIndex) -> TxToken {
        TxToken {
            buffer: tx_buffer,
            shared_inner: self.shared_inner().clone(),
        }
    }
}

impl Device for DeviceImpl {
    type RxToken<'a> = RxToken;
    type TxToken<'a> = TxToken;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut cap = DeviceCapabilities::default();
        cap.max_transmission_unit = self.shared_inner().borrow().mtu();
        cap
    }

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let r = self.shared_inner().borrow_mut().receive();
        r.ok()
            .map(|(rx_ix, tx_ix)| (self.new_rx_token(rx_ix), self.new_tx_token(tx_ix)))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        self.shared_inner()
            .borrow_mut()
            .transmit()
            .ok()
            .map(|ix| self.new_tx_token(ix))
    }
}

pub struct RxToken {
    buffer: RxBufferIndex,
    shared_inner: SharedInner,
}

impl phy::RxToken for RxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        // let r = self.shared_inner.borrow_mut().consume_rx(self.buffer, f);
        let ptr = self.shared_inner.borrow_mut().consume_rx_start(self.buffer);
        let r = f(unsafe { ptr.as_mut().unwrap() });
        drop(self);
        r
    }
}

impl Drop for RxToken {
    fn drop(&mut self) {
        self.shared_inner.borrow_mut().drop_rx(self.buffer)
    }
}

pub struct TxToken {
    buffer: TxBufferIndex,
    shared_inner: SharedInner,
}

impl phy::TxToken for TxToken {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let r = self
            .shared_inner
            .borrow_mut()
            .consume_tx(self.buffer, len, f);
        drop(self);
        r
    }
}

impl Drop for TxToken {
    fn drop(&mut self) {
        self.shared_inner.borrow_mut().drop_tx(self.buffer)
    }
}
