//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::alloc::Layout;
use core::ptr::{self, NonNull};

use one_shot_mutex::sync::OneShotMutex;
use virtio_drivers::{BufferDirection, Hal, PAGE_SIZE, PhysAddr};

use sel4_abstract_allocator::basic::BasicAllocator;
use sel4_abstract_allocator::{ByRange, WithAlignmentBound};
use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;
use sel4_shared_memory::SharedMemoryRef;

static GLOBAL_STATE: ImmediateSyncOnceCell<OneShotMutex<State>> = ImmediateSyncOnceCell::new();

struct State {
    dma_region: SharedMemoryRef<'static, [u8]>,
    dma_region_paddr: usize,
    bounce_buffer_allocator: ByRange<WithAlignmentBound<BasicAllocator>>,
}

impl State {
    fn offset_to_paddr(&self, offset: usize) -> PhysAddr {
        self.dma_region_paddr
            .checked_add(offset)
            .unwrap()
            .try_into()
            .unwrap()
    }

    fn paddr_to_offset(&self, paddr: PhysAddr) -> usize {
        usize::try_from(paddr)
            .unwrap()
            .checked_sub(self.dma_region_paddr)
            .unwrap()
    }
}

pub struct HalImpl;

impl HalImpl {
    pub fn init(dma_region_size: usize, dma_region_vaddr: usize, dma_region_paddr: usize) {
        let dma_region_ptr = NonNull::new(ptr::slice_from_raw_parts_mut(
            dma_region_vaddr as *mut _,
            dma_region_size,
        ))
        .unwrap();

        let dma_region = unsafe { SharedMemoryRef::new(dma_region_ptr) };

        let max_alignment = 1
            << dma_region_vaddr
                .trailing_zeros()
                .min(dma_region_paddr.trailing_zeros());

        let bounce_buffer_allocator = ByRange::new(WithAlignmentBound::new(
            BasicAllocator::new(dma_region_size),
            max_alignment,
        ));

        GLOBAL_STATE
            .set(OneShotMutex::new(State {
                dma_region,
                dma_region_paddr,
                bounce_buffer_allocator,
            }))
            .ok()
            .unwrap();
    }
}

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let mut state = GLOBAL_STATE.get().unwrap().lock();
        assert!(pages > 0);
        let layout = Layout::from_size_align(pages * PAGE_SIZE, PAGE_SIZE).unwrap();
        let bounce_buffer_range = state.bounce_buffer_allocator.allocate(layout).unwrap();
        let bounce_buffer_ptr = state
            .dma_region
            .as_mut_ptr()
            .index(bounce_buffer_range.clone());
        bounce_buffer_ptr.fill(0);
        let vaddr = bounce_buffer_ptr.as_raw_ptr().cast::<u8>();
        let paddr = state.offset_to_paddr(bounce_buffer_range.start);
        (paddr, vaddr)
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        let mut state = GLOBAL_STATE.get().unwrap().lock();
        let bounce_buffer_range = {
            let start = state.paddr_to_offset(paddr);
            let size = pages * PAGE_SIZE;
            start..(start + size)
        };
        state
            .bounce_buffer_allocator
            .deallocate(bounce_buffer_range);
        0
    }

    unsafe fn mmio_phys_to_virt(_paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        panic!()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        let mut state = GLOBAL_STATE.get().unwrap().lock();
        assert!(!buffer.is_empty());
        let layout = Layout::from_size_align(buffer.len(), 1).unwrap();
        let bounce_buffer_range = state.bounce_buffer_allocator.allocate(layout).unwrap();
        let buffer_slice = unsafe { buffer.as_ref() };
        state
            .dma_region
            .as_mut_ptr()
            .index(bounce_buffer_range.clone())
            .copy_from_slice(buffer_slice);
        state.offset_to_paddr(bounce_buffer_range.start)
    }

    unsafe fn unshare(paddr: PhysAddr, mut buffer: NonNull<[u8]>, direction: BufferDirection) {
        let mut state = GLOBAL_STATE.get().unwrap().lock();
        let bounce_buffer_range = {
            let start = state.paddr_to_offset(paddr);
            start..(start + buffer.len())
        };
        if direction != BufferDirection::DriverToDevice {
            let buffer_slice = unsafe { buffer.as_mut() };
            state
                .dma_region
                .as_mut_ptr()
                .index(bounce_buffer_range.clone())
                .copy_into_slice(buffer_slice);
        }
        state
            .bounce_buffer_allocator
            .deallocate(bounce_buffer_range);
    }
}
