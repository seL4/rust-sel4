#![no_std]
#![feature(never_type)]
#![feature(strict_provenance)]

extern crate alloc;

use alloc::rc::Rc;
use alloc::vec::Vec;
use core::alloc::Layout;
use core::cell::RefCell;
use core::iter;
use core::ops::Range;

use smoltcp::phy::{self, Device, DeviceCapabilities};
use smoltcp::time::Instant;

use sel4_bounce_buffer_allocator::{Basic, BounceBufferAllocator};
use sel4_externally_shared::ExternallySharedRef;
use sel4_shared_ring_buffer::{
    Descriptor, Error as SharedRingBuffersError, RingBuffer, RingBuffers,
};

pub struct DeviceImpl {
    shared_driver: SharedDriver,
}

type SharedDriver = Rc<RefCell<SharedDriverInner>>;

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
            shared_driver: Rc::new(RefCell::new(SharedDriverInner::new(
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

    fn shared_driver(&self) -> &SharedDriver {
        &self.shared_driver
    }

    pub fn handle_notification(&self) {
        self.shared_driver().borrow_mut().handle_notification()
    }

    fn new_rx_token(&self, rx_buffer: RxBufferIndex) -> RxToken {
        RxToken {
            buffer: rx_buffer,
            shared_driver: self.shared_driver().clone(),
        }
    }

    fn new_tx_token(&self, tx_buffer: TxBufferIndex) -> TxToken {
        TxToken {
            buffer: tx_buffer,
            shared_driver: self.shared_driver().clone(),
        }
    }
}

impl Device for DeviceImpl {
    type RxToken<'a> = RxToken;
    type TxToken<'a> = TxToken;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut cap = DeviceCapabilities::default();
        cap.max_transmission_unit = self.shared_driver().borrow().mtu();
        cap
    }

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let r = self.shared_driver().borrow_mut().receive();
        r.ok()
            .map(|(rx_ix, tx_ix)| (self.new_rx_token(rx_ix), self.new_tx_token(tx_ix)))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        self.shared_driver()
            .borrow_mut()
            .transmit()
            .ok()
            .map(|ix| self.new_tx_token(ix))
    }
}

pub struct RxToken {
    buffer: RxBufferIndex,
    shared_driver: SharedDriver,
}

impl phy::RxToken for RxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        // let r = self.shared_driver.borrow_mut().consume_rx(self.buffer, f);
        let ptr = self
            .shared_driver
            .borrow_mut()
            .consume_rx_start(self.buffer);
        let r = f(unsafe { ptr.as_mut().unwrap() });
        drop(self);
        r
    }
}

impl Drop for RxToken {
    fn drop(&mut self) {
        self.shared_driver.borrow_mut().drop_rx(self.buffer)
    }
}

pub struct TxToken {
    buffer: TxBufferIndex,
    shared_driver: SharedDriver,
}

impl phy::TxToken for TxToken {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let r = self
            .shared_driver
            .borrow_mut()
            .consume_tx(self.buffer, len, f);
        drop(self);
        r
    }
}

impl Drop for TxToken {
    fn drop(&mut self) {
        self.shared_driver.borrow_mut().drop_tx(self.buffer)
    }
}

struct SharedDriverInner {
    dma_region: ExternallySharedRef<'static, [u8]>,
    dma_region_paddr: usize,
    bounce_buffer_allocator: BounceBufferAllocator<Basic>,
    rx_ring_buffers: RingBuffers<'static, fn() -> Result<(), !>>,
    tx_ring_buffers: RingBuffers<'static, fn() -> Result<(), !>>,
    rx_buffers: Vec<RxBufferEntry>,
    tx_buffers: Vec<TxBufferEntry>,
    mtu: usize,
}

type RxBufferIndex = usize;

#[derive(Clone, Debug, Eq, PartialEq)]
struct RxBufferEntry {
    state: RxBufferState,
    range: Range<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RxBufferState {
    Free,
    Used { len: usize },
    Claimed { len: usize },
}

type TxBufferIndex = usize;

#[derive(Clone, Debug, Eq, PartialEq)]
struct TxBufferEntry {
    state: TxBufferState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TxBufferState {
    Unclaimed,
    SlotClaimed,
    Sent { range: Range<usize> },
}

impl SharedDriverInner {
    fn new(
        dma_region: ExternallySharedRef<'static, [u8]>,
        dma_region_paddr: usize,
        mut rx_ring_buffers: RingBuffers<'static, fn() -> Result<(), !>>,
        tx_ring_buffers: RingBuffers<'static, fn() -> Result<(), !>>,
        num_rx_buffers: usize,
        rx_buffer_size: usize,
        mtu: usize,
    ) -> Self {
        let max_alignment = 1
            << dma_region
                .as_ptr()
                .as_raw_ptr()
                .addr()
                .trailing_zeros()
                .min(dma_region_paddr.trailing_zeros());

        let mut bounce_buffer_allocator =
            BounceBufferAllocator::new(Basic::new(dma_region.as_ptr().len()), max_alignment);

        let rx_buffers = iter::repeat_with(|| RxBufferEntry {
            state: RxBufferState::Free,
            range: bounce_buffer_allocator
                .allocate(Layout::from_size_align(rx_buffer_size, 1).unwrap())
                .unwrap(),
        })
        .take(num_rx_buffers)
        .collect::<Vec<_>>();

        for entry in rx_buffers.iter() {
            rx_ring_buffers
                .free_mut()
                .enqueue(descriptor_of(dma_region_paddr, entry.range.clone()))
                .unwrap();
        }
        let tx_buffers = iter::repeat_with(|| TxBufferEntry {
            state: TxBufferState::Unclaimed,
        })
        .take(RingBuffer::SIZE)
        .collect::<Vec<_>>();

        Self {
            dma_region,
            dma_region_paddr,
            bounce_buffer_allocator,
            rx_ring_buffers,
            tx_ring_buffers,
            rx_buffers,
            tx_buffers,
            mtu,
        }
    }

    fn mtu(&self) -> usize {
        self.mtu
    }

    fn handle_notification(&mut self) {
        while let Some(desc) = self
            .rx_ring_buffers
            .used_mut()
            .dequeue()
            .map_err(|err| assert_eq!(err, SharedRingBuffersError::RingIsEmpty))
            .ok()
        {
            let ix = self
                .lookup_rx_buffer_by_encoded_addr(desc.encoded_addr())
                .unwrap();
            let entry = self.rx_buffer_entry_mut(ix);
            assert_eq!(entry.state, RxBufferState::Free);
            entry.state = RxBufferState::Used {
                len: desc.len().try_into().unwrap(),
            };
            self.rx_ring_buffers.notify().unwrap();
        }

        while let Some(desc) = self
            .tx_ring_buffers
            .used_mut()
            .dequeue()
            .map_err(|err| assert_eq!(err, SharedRingBuffersError::RingIsEmpty))
            .ok()
        {
            let ix = self.lookup_tx_buffer_by_descriptor(&desc).unwrap();
            let entry = self.tx_buffer_entry_mut(ix);
            let range = match &entry.state {
                TxBufferState::Sent { range } => range.clone(),
                _ => {
                    panic!()
                }
            };
            entry.state = TxBufferState::Unclaimed;
            self.bounce_buffer_allocator.deallocate(range);
            self.tx_ring_buffers.notify().unwrap();
        }
    }

    fn lookup_rx_buffer_by_encoded_addr(&self, encoded_addr: usize) -> Option<RxBufferIndex> {
        self.rx_buffers
            .iter()
            .enumerate()
            .find(|(_i, entry)| self.dma_region_paddr + entry.range.start == encoded_addr)
            .map(|(i, _entry)| i)
    }

    fn lookup_tx_buffer_by_descriptor(&self, desc: &Descriptor) -> Option<TxBufferIndex> {
        self.tx_buffers
            .iter()
            .enumerate()
            .find(|(_i, entry)| match &entry.state {
                TxBufferState::Sent { range } => {
                    &descriptor_of(self.dma_region_paddr, range.clone()) == desc
                }
                _ => false,
            })
            .map(|(i, _entry)| i)
    }

    fn rx_buffer_entry_mut(&mut self, index: RxBufferIndex) -> &mut RxBufferEntry {
        &mut self.rx_buffers[index]
    }

    fn tx_buffer_entry_mut(&mut self, index: TxBufferIndex) -> &mut TxBufferEntry {
        &mut self.tx_buffers[index]
    }

    fn receive(&mut self) -> Result<(RxBufferIndex, TxBufferIndex), ()> {
        if let (Some(rx), Some(tx)) = (self.can_claim_rx_buffer(), self.can_claim_tx_buffer()) {
            self.claim_rx_buffer(rx);
            self.claim_tx_buffer(tx);
            Ok((rx, tx))
        } else {
            Err(())
        }
    }

    fn transmit(&mut self) -> Result<TxBufferIndex, ()> {
        self.can_claim_tx_buffer()
            .map(|index| {
                self.claim_tx_buffer(index);
                index
            })
            .ok_or(())
    }

    fn can_claim_rx_buffer(&self) -> Option<RxBufferIndex> {
        self.rx_buffers
            .iter()
            .enumerate()
            .find(|(_i, entry)| matches!(entry.state, RxBufferState::Used { .. }))
            .map(|(i, _entry)| i)
    }

    fn claim_rx_buffer(&mut self, index: RxBufferIndex) {
        let entry = self.rx_buffer_entry_mut(index);
        let len = match entry.state {
            RxBufferState::Used { len } => len,
            _ => panic!(),
        };
        entry.state = RxBufferState::Claimed { len };
    }

    fn can_claim_tx_buffer(&self) -> Option<TxBufferIndex> {
        self.tx_buffers
            .iter()
            .enumerate()
            .find(|(_i, entry)| entry.state == TxBufferState::Unclaimed)
            .map(|(i, _entry)| i)
    }

    fn claim_tx_buffer(&mut self, index: TxBufferIndex) {
        let entry = self.tx_buffer_entry_mut(index);
        assert_eq!(entry.state, TxBufferState::Unclaimed);
        entry.state = TxBufferState::SlotClaimed;
    }

    fn consume_rx_start(&mut self, index: RxBufferIndex) -> *mut [u8] {
        let entry = self.rx_buffer_entry_mut(index);
        let range = entry.range.clone();
        let len = match entry.state {
            RxBufferState::Claimed { len } => len,
            _ => panic!(),
        };
        unsafe {
            self.dma_region
                .as_mut_ptr()
                .index(range.start..range.start + len)
                .as_raw_ptr()
                .as_mut()
        }
    }

    fn drop_rx(&mut self, index: RxBufferIndex) {
        let entry = self.rx_buffer_entry_mut(index);
        let state = entry.state.clone();
        match state {
            RxBufferState::Used { .. } => {}
            RxBufferState::Claimed { .. } => {
                entry.state = RxBufferState::Free;
                let range = entry.range.clone();
                let desc = descriptor_of(self.dma_region_paddr, range);
                self.rx_ring_buffers.free_mut().enqueue(desc).unwrap();
                self.rx_ring_buffers.notify().unwrap();
            }
            _ => {
                panic!()
            }
        }
    }

    fn consume_tx<F, R>(&mut self, index: TxBufferIndex, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let range = self
            .bounce_buffer_allocator
            .allocate(Layout::from_size_align(len, 1).unwrap())
            .unwrap();
        let entry = self.tx_buffer_entry_mut(index);
        assert_eq!(entry.state, TxBufferState::SlotClaimed);
        entry.state = TxBufferState::Sent {
            range: range.clone(),
        };
        let r = f(unsafe {
            self.dma_region
                .as_mut_ptr()
                .index(range.clone())
                .as_raw_ptr()
                .as_mut()
        });
        let desc = descriptor_of(self.dma_region_paddr, range);
        self.tx_ring_buffers.free_mut().enqueue(desc).unwrap();
        self.tx_ring_buffers.notify().unwrap();
        r
    }

    fn drop_tx(&mut self, index: TxBufferIndex) {
        let entry = self.tx_buffer_entry_mut(index);
        let state = entry.state.clone();
        match state {
            TxBufferState::SlotClaimed => {
                entry.state = TxBufferState::Unclaimed;
            }
            TxBufferState::Sent { .. } => {}
            _ => {
                panic!()
            }
        }
    }
}

fn descriptor_of(dma_region_paddr: usize, range: Range<usize>) -> Descriptor {
    Descriptor::new(
        dma_region_paddr + range.start,
        range.len().try_into().unwrap(),
        0,
    )
}
