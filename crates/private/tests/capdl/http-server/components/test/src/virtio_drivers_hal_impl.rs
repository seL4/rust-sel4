use core::alloc::Layout;
use core::ptr::NonNull;

use virtio_drivers::{BufferDirection, Hal, PhysAddr, PAGE_SIZE};

use sel4_bounce_buffer_allocator::{Basic, BounceBufferAllocator};
use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;
use sel4_sync::{GenericMutex, PanickingMutexSyncOps};

const MAX_ALIGNMENT: usize = 4096;

static DMA_VADDR_TO_PADDR_OFFSET: ImmediateSyncOnceCell<isize> = ImmediateSyncOnceCell::new();

static BOUNCE_BUFFER_ALLOCATOR: GenericMutex<
    PanickingMutexSyncOps,
    Option<BounceBufferAllocator<Basic>>,
> = GenericMutex::new(PanickingMutexSyncOps::new(), None);

pub struct HalImpl;

impl HalImpl {
    pub(crate) fn init(dma_region: NonNull<[u8]>, dma_vaddr_to_paddr_offset: isize) {
        DMA_VADDR_TO_PADDR_OFFSET
            .set(dma_vaddr_to_paddr_offset)
            .unwrap();

        {
            let mut lock = BOUNCE_BUFFER_ALLOCATOR.lock();
            *lock = unsafe {
                Some(BounceBufferAllocator::new(
                    Basic::new(dma_region.len()),
                    dma_region,
                    MAX_ALIGNMENT,
                ))
            };
        }
    }
}

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        assert!(pages > 0);
        let layout = Layout::from_size_align(pages * PAGE_SIZE, PAGE_SIZE).unwrap();
        // Safe because the layout has a non-zero size.
        let ptr = {
            let mut lock = BOUNCE_BUFFER_ALLOCATOR.lock();
            lock.as_mut()
                .unwrap()
                .allocate_zeroed(layout)
                .unwrap()
                .as_non_null_ptr()
        };
        let paddr = virt_to_phys(ptr.addr().get());
        (paddr, ptr)
    }

    unsafe fn dma_dealloc(_paddr: PhysAddr, vaddr: NonNull<u8>, pages: usize) -> i32 {
        {
            let mut lock = BOUNCE_BUFFER_ALLOCATOR.lock();
            lock.as_mut().unwrap().deallocate(vaddr, pages * PAGE_SIZE);
        }
        0
    }

    unsafe fn mmio_phys_to_virt(_paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        panic!()
    }

    unsafe fn share(buffer: NonNull<[u8]>, direction: BufferDirection) -> PhysAddr {
        assert!(buffer.len() > 0);
        let layout = Layout::from_size_align(buffer.len(), 1).unwrap();
        // Safe because the layout has a non-zero size.
        let mut bounce_buffer_ptr = {
            let mut lock = BOUNCE_BUFFER_ALLOCATOR.lock();
            let allocator = lock.as_mut().unwrap();
            if direction != BufferDirection::DriverToDevice {
                allocator.allocate_zeroed(layout)
            } else {
                allocator.allocate(layout)
            }
            .unwrap()
        };
        if direction != BufferDirection::DeviceToDriver {
            unsafe {
                bounce_buffer_ptr.as_mut().copy_from_slice(buffer.as_ref());
            }
        }
        let paddr = virt_to_phys(bounce_buffer_ptr.addr().get());
        paddr
    }

    unsafe fn unshare(paddr: PhysAddr, mut buffer: NonNull<[u8]>, direction: BufferDirection) {
        let bounce_buffer_ptr = NonNull::slice_from_raw_parts(
            NonNull::new(phys_to_virt(paddr) as *mut _).unwrap(),
            buffer.len(),
        );
        if direction != BufferDirection::DriverToDevice {
            unsafe {
                buffer.as_mut().copy_from_slice(bounce_buffer_ptr.as_ref());
            }
        }
        {
            let mut lock = BOUNCE_BUFFER_ALLOCATOR.lock();
            lock.as_mut()
                .unwrap()
                .deallocate(bounce_buffer_ptr.as_non_null_ptr(), bounce_buffer_ptr.len());
        }
    }
}

fn virt_to_phys(vaddr: usize) -> PhysAddr {
    usize::try_from(isize::try_from(vaddr).unwrap() + DMA_VADDR_TO_PADDR_OFFSET.get().unwrap())
        .unwrap()
}

fn phys_to_virt(paddr: PhysAddr) -> usize {
    usize::try_from(isize::try_from(paddr).unwrap() - DMA_VADDR_TO_PADDR_OFFSET.get().unwrap())
        .unwrap()
}
