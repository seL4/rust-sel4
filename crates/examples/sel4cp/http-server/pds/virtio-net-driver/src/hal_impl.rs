use core::alloc::Layout;
use core::ops::Range;
use core::ptr::{self, NonNull};

use virtio_drivers::{BufferDirection, Hal, PhysAddr, PAGE_SIZE};

use sel4_bounce_buffer_allocator::{Basic, BounceBufferAllocator};
use sel4_externally_shared::{ExternallySharedPtr, ExternallySharedRef};
use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;
use sel4_sync::{GenericMutex, PanickingMutexSyncOps};

const MAX_ALIGNMENT: usize = 4096;

static DMA_REGION_VADDR_RANGE: ImmediateSyncOnceCell<Range<usize>> = ImmediateSyncOnceCell::new();

static DMA_REGION_PADDR: ImmediateSyncOnceCell<usize> = ImmediateSyncOnceCell::new();

static BOUNCE_BUFFER_ALLOCATOR: GenericMutex<
    PanickingMutexSyncOps,
    Option<BounceBufferAllocator<Basic>>,
> = GenericMutex::new(PanickingMutexSyncOps::new(), None);

pub(crate) struct HalImpl;

impl HalImpl {
    pub(crate) fn init(dma_region_size: usize, dma_region_vaddr: usize, dma_region_paddr: usize) {
        DMA_REGION_VADDR_RANGE
            .set(dma_region_vaddr..(dma_region_vaddr + dma_region_size))
            .unwrap();

        DMA_REGION_PADDR.set(dma_region_paddr).unwrap();

        {
            let mut lock = BOUNCE_BUFFER_ALLOCATOR.lock();
            *lock = Some(BounceBufferAllocator::new(
                Basic::new(dma_region_size),
                MAX_ALIGNMENT,
            ));
        }
    }
}

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        assert!(pages > 0);
        let layout = Layout::from_size_align(pages * PAGE_SIZE, PAGE_SIZE).unwrap();
        let buffer = {
            let mut lock = BOUNCE_BUFFER_ALLOCATOR.lock();
            lock.as_mut().unwrap().allocate(layout).unwrap()
        };
        let vaddr = with_bounce_buffer_ptr(buffer.clone(), |ptr| {
            ptr.fill(0);
            ptr.as_raw_ptr().as_non_null_ptr()
        });
        let paddr = offset_to_paddr(buffer.start);
        (paddr, vaddr)
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        let buffer = {
            let start = paddr_to_offset(paddr);
            let size = pages * PAGE_SIZE;
            start..(start + size)
        };
        {
            let mut lock = BOUNCE_BUFFER_ALLOCATOR.lock();
            lock.as_mut().unwrap().deallocate(buffer);
        }
        0
    }

    unsafe fn mmio_phys_to_virt(_paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        panic!()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        assert!(buffer.len() > 0);
        let layout = Layout::from_size_align(buffer.len(), 1).unwrap();
        let bounce_buffer = {
            let mut lock = BOUNCE_BUFFER_ALLOCATOR.lock();
            lock.as_mut().unwrap().allocate(layout).unwrap()
        };
        with_bounce_buffer_ptr(bounce_buffer.clone(), |ptr| {
            ptr.copy_from_slice(buffer.as_ref());
        });
        let paddr = offset_to_paddr(bounce_buffer.start);
        paddr
    }

    unsafe fn unshare(paddr: PhysAddr, mut buffer: NonNull<[u8]>, direction: BufferDirection) {
        let bounce_buffer = {
            let start = paddr_to_offset(paddr);
            start..(start + buffer.len())
        };
        if direction != BufferDirection::DriverToDevice {
            with_bounce_buffer_ptr(bounce_buffer.clone(), |ptr| {
                ptr.copy_into_slice(buffer.as_mut());
            });
        }
        {
            let mut lock = BOUNCE_BUFFER_ALLOCATOR.lock();
            lock.as_mut().unwrap().deallocate(bounce_buffer);
        }
    }
}

fn with_bounce_buffer_ptr<F, R>(bounce_buffer: Range<usize>, f: F) -> R
where
    F: FnOnce(ExternallySharedPtr<'_, [u8]>) -> R,
{
    f(dma_region().as_mut_ptr().index(bounce_buffer))
}

fn dma_region() -> ExternallySharedRef<'static, [u8]> {
    let vaddr_range = DMA_REGION_VADDR_RANGE.get().unwrap();
    let ptr = NonNull::new(ptr::from_raw_parts_mut(
        ptr::from_exposed_addr_mut(vaddr_range.start),
        vaddr_range.len(),
    ))
    .unwrap();
    unsafe { ExternallySharedRef::new(ptr) }
}

fn offset_to_paddr(offset: usize) -> PhysAddr {
    DMA_REGION_PADDR.get().unwrap() + offset
}

fn paddr_to_offset(paddr: PhysAddr) -> usize {
    paddr.checked_sub(*DMA_REGION_PADDR.get().unwrap()).unwrap()
}
