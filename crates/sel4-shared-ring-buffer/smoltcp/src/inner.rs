use alloc::vec::Vec;
use core::alloc::Layout;
use core::iter;
use core::ops::Range;
use core::ptr::NonNull;

use sel4_bounce_buffer_allocator::{AbstractBounceBufferAllocator, BounceBufferAllocator};
use sel4_externally_shared::ExternallySharedRef;
use sel4_shared_ring_buffer::{roles::Provide, Descriptor, RingBuffers, RING_BUFFER_SIZE};

pub(crate) struct Inner<A> {
    dma_region: ExternallySharedRef<'static, [u8]>,
    bounce_buffer_allocator: BounceBufferAllocator<A>,
    rx_ring_buffers: RingBuffers<'static, Provide, fn()>,
    tx_ring_buffers: RingBuffers<'static, Provide, fn()>,
    rx_buffers: Vec<RxBufferEntry>,
    tx_buffers: Vec<TxBufferEntry>,
    mtu: usize,
}

pub(crate) type RxBufferIndex = usize;

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

pub(crate) type TxBufferIndex = usize;

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

impl<A: AbstractBounceBufferAllocator> Inner<A> {
    pub(crate) fn new(
        dma_region: ExternallySharedRef<'static, [u8]>,
        mut bounce_buffer_allocator: BounceBufferAllocator<A>,
        mut rx_ring_buffers: RingBuffers<'static, Provide, fn()>,
        tx_ring_buffers: RingBuffers<'static, Provide, fn()>,
        num_rx_buffers: usize,
        rx_buffer_size: usize,
        mtu: usize,
    ) -> Self {
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
                .enqueue_and_commit(descriptor_of(entry.range.clone()))
                .unwrap()
                .unwrap();
        }
        let tx_buffers = iter::repeat_with(|| TxBufferEntry {
            state: TxBufferState::Unclaimed,
        })
        .take(RING_BUFFER_SIZE)
        .collect::<Vec<_>>();

        Self {
            dma_region,
            bounce_buffer_allocator,
            rx_ring_buffers,
            tx_ring_buffers,
            rx_buffers,
            tx_buffers,
            mtu,
        }
    }

    pub(crate) fn mtu(&self) -> usize {
        self.mtu
    }

    pub(crate) fn poll(&mut self) -> bool {
        let mut notify_rx = false;

        while let Some(desc) = self.rx_ring_buffers.used_mut().dequeue().unwrap() {
            let ix = self
                .lookup_rx_buffer_by_encoded_addr(desc.encoded_addr())
                .unwrap();
            let entry = self.rx_buffer_entry_mut(ix);
            assert_eq!(entry.state, RxBufferState::Free);
            entry.state = RxBufferState::Used {
                len: desc.len().try_into().unwrap(),
            };
            notify_rx = true;
        }

        if notify_rx {
            self.rx_ring_buffers.notify();
        }

        let mut notify_tx = false;

        while let Some(desc) = self.tx_ring_buffers.used_mut().dequeue().unwrap() {
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
            notify_tx = true;
        }

        if notify_tx {
            self.tx_ring_buffers.notify();
        }

        notify_rx || notify_tx
    }

    fn lookup_rx_buffer_by_encoded_addr(&self, encoded_addr: usize) -> Option<RxBufferIndex> {
        self.rx_buffers
            .iter()
            .enumerate()
            .find(|(_i, entry)| entry.range.start == encoded_addr)
            .map(|(i, _entry)| i)
    }

    fn lookup_tx_buffer_by_descriptor(&self, desc: &Descriptor) -> Option<TxBufferIndex> {
        self.tx_buffers
            .iter()
            .enumerate()
            .find(|(_i, entry)| match &entry.state {
                TxBufferState::Sent { range } => &descriptor_of(range.clone()) == desc,
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

    pub(crate) fn receive(&mut self) -> Result<(RxBufferIndex, TxBufferIndex), ()> {
        if let (Some(rx), Some(tx)) = (self.can_claim_rx_buffer(), self.can_claim_tx_buffer()) {
            self.claim_rx_buffer(rx);
            self.claim_tx_buffer(tx);
            Ok((rx, tx))
        } else {
            Err(())
        }
    }

    pub(crate) fn transmit(&mut self) -> Result<TxBufferIndex, ()> {
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

    pub(crate) fn consume_rx_start(&mut self, index: RxBufferIndex) -> NonNull<[u8]> {
        let entry = self.rx_buffer_entry_mut(index);
        let range = entry.range.clone();
        let len = match entry.state {
            RxBufferState::Claimed { len } => len,
            _ => panic!(),
        };
        self.dma_region
            .as_mut_ptr()
            .index(range.start..range.start + len)
            .as_raw_ptr()
    }

    pub(crate) fn drop_rx(&mut self, index: RxBufferIndex) {
        let entry = self.rx_buffer_entry_mut(index);
        let state = entry.state.clone();
        match state {
            RxBufferState::Used { .. } => {}
            RxBufferState::Claimed { .. } => {
                entry.state = RxBufferState::Free;
                let range = entry.range.clone();
                let desc = descriptor_of(range);
                self.rx_ring_buffers
                    .free_mut()
                    .enqueue_and_commit(desc)
                    .unwrap()
                    .unwrap();
                self.rx_ring_buffers.notify();
            }
            _ => {
                panic!()
            }
        }
    }

    pub(crate) fn consume_tx<F, R>(&mut self, index: TxBufferIndex, len: usize, f: F) -> R
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
        let mut ptr = self
            .dma_region
            .as_mut_ptr()
            .index(range.clone())
            .as_raw_ptr();
        let r = f(unsafe { ptr.as_mut() });
        let desc = descriptor_of(range);
        self.tx_ring_buffers
            .free_mut()
            .enqueue_and_commit(desc)
            .unwrap()
            .unwrap();
        self.tx_ring_buffers.notify();
        r
    }

    pub(crate) fn drop_tx(&mut self, index: TxBufferIndex) {
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

fn descriptor_of(range: Range<usize>) -> Descriptor {
    Descriptor::from_encoded_addr_range(range, 0)
}
