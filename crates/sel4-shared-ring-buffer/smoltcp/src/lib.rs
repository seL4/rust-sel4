//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

extern crate alloc;

use lock_api::{Mutex, RawMutex};
use one_shot_mutex::unsync::RawOneShotMutex;
use smoltcp::phy::{self, Device, DeviceCapabilities};
use smoltcp::time::Instant;

use sel4_abstract_allocator::AbstractAllocator;
use sel4_abstract_rc::{AbstractRcT, RcT};
use sel4_shared_memory::SharedMemoryRef;
use sel4_shared_ring_buffer::{RingBuffers, roles::Provide};

mod inner;

pub use inner::{Error, PeerMisbehaviorError};
use inner::{Inner, RxBufferIndex, TxBufferIndex};

pub struct DeviceImpl<A: AbstractAllocator, R: RawMutex = RawOneShotMutex, P: AbstractRcT = RcT> {
    inner: P::Rc<Mutex<R, Inner<A>>>,
}

impl<A: AbstractAllocator, R: RawMutex, P: AbstractRcT> Clone for DeviceImpl<A, R, P> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<A: AbstractAllocator, R: RawMutex, P: AbstractRcT> DeviceImpl<A, R, P> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        raw_mutex: R,
        dma_region: SharedMemoryRef<'static, [u8]>,
        bounce_buffer_allocator: A,
        rx_ring_buffers: RingBuffers<'static, Provide, fn()>,
        tx_ring_buffers: RingBuffers<'static, Provide, fn()>,
        num_rx_buffers: usize,
        rx_buffer_size: usize,
        caps: DeviceCapabilities,
    ) -> Result<Self, Error> {
        Ok(Self {
            inner: P::Rc::from(Mutex::from_raw(
                raw_mutex,
                Inner::new(
                    dma_region,
                    bounce_buffer_allocator,
                    rx_ring_buffers,
                    tx_ring_buffers,
                    num_rx_buffers,
                    rx_buffer_size,
                    caps,
                )?,
            )),
        })
    }

    fn inner(&self) -> &P::Rc<Mutex<R, Inner<A>>> {
        &self.inner
    }

    pub fn poll(&self) -> bool {
        self.inner().lock().poll().unwrap()
    }

    pub fn can_receive(&self) -> bool {
        self.inner().lock().can_receive()
    }

    pub fn can_transmit(&self) -> bool {
        self.inner().lock().can_transmit()
    }

    fn new_rx_token(&self, rx_buffer: RxBufferIndex) -> RxToken<A, R, P> {
        RxToken {
            buffer: rx_buffer,
            shared: self.clone(),
        }
    }

    fn new_tx_token(&self, tx_buffer: TxBufferIndex) -> TxToken<A, R, P> {
        TxToken {
            buffer: tx_buffer,
            shared: self.clone(),
        }
    }
}

impl<A: AbstractAllocator, R: RawMutex, P: AbstractRcT> Device for DeviceImpl<A, R, P> {
    type RxToken<'a>
        = RxToken<A, R, P>
    where
        A: 'a,
        R: 'a,
        P: 'a;
    type TxToken<'a>
        = TxToken<A, R, P>
    where
        A: 'a,
        R: 'a,
        P: 'a;

    fn capabilities(&self) -> DeviceCapabilities {
        self.inner().lock().caps().clone()
    }

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        self.inner()
            .lock()
            .receive()
            .map(|(rx_ix, tx_ix)| (self.new_rx_token(rx_ix), self.new_tx_token(tx_ix)))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        self.inner()
            .lock()
            .transmit()
            .map(|ix| self.new_tx_token(ix))
    }
}

pub struct RxToken<A: AbstractAllocator, R: RawMutex, P: AbstractRcT> {
    buffer: RxBufferIndex,
    shared: DeviceImpl<A, R, P>,
}

impl<A: AbstractAllocator, R: RawMutex, P: AbstractRcT> phy::RxToken for RxToken<A, R, P> {
    fn consume<T, F>(self, f: F) -> T
    where
        F: FnOnce(&mut [u8]) -> T,
    {
        let mut ptr = self.shared.inner().lock().consume_rx_start(self.buffer);
        let r = f(unsafe { ptr.as_mut() });
        self.shared.inner().lock().consume_rx_finish(self.buffer);
        r
    }
}

impl<A: AbstractAllocator, R: RawMutex, P: AbstractRcT> Drop for RxToken<A, R, P> {
    fn drop(&mut self) {
        self.shared.inner().lock().drop_rx(self.buffer).unwrap()
    }
}

pub struct TxToken<A: AbstractAllocator, R: RawMutex, P: AbstractRcT> {
    buffer: TxBufferIndex,
    shared: DeviceImpl<A, R, P>,
}

impl<A: AbstractAllocator, R: RawMutex, P: AbstractRcT> phy::TxToken for TxToken<A, R, P> {
    fn consume<T, F>(self, len: usize, f: F) -> T
    where
        F: FnOnce(&mut [u8]) -> T,
    {
        self.shared
            .inner()
            .lock()
            .consume_tx(self.buffer, len, f)
            .unwrap()
    }
}

impl<A: AbstractAllocator, R: RawMutex, P: AbstractRcT> Drop for TxToken<A, R, P> {
    fn drop(&mut self) {
        self.shared.inner().lock().drop_tx(self.buffer)
    }
}
