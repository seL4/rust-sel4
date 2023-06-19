use core::alloc::Layout;
use core::ptr::NonNull;

use log::trace;
use virtio_drivers::{BufferDirection, Hal, PhysAddr, PAGE_SIZE};

use sel4_bounce_buffer_allocator::{BounceBufferAllocator, Bump};
use sel4_sync::{GenericMutex, PanickingMutexSyncOps};

const MAX_ALIGNMENT: usize = 4096;

pub struct HalImpl;

impl HalImpl {
    pub(crate) fn init(dma_region: NonNull<[u8]>, dma_vaddr_to_paddr_offset: isize) {
        {
            let mut lock = DMA_VADDR_TO_PADDR_OFFSET.lock();
            *lock = Some(dma_vaddr_to_paddr_offset);
        }

        {
            let mut lock = BOUNCE_BUFFER_ALLOCATOR.lock();
            *lock = unsafe {
                Some(BounceBufferAllocator::new(
                    Bump::new(dma_region.len()),
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
        trace!(
            "alloc DMA: paddr={:#x}, vaddr={:#x?}, pages={}",
            paddr,
            ptr,
            pages
        );
        (paddr, ptr)
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, vaddr: NonNull<u8>, pages: usize) -> i32 {
        trace!("dealloc DMA: paddr={:#x}, pages={}", paddr, pages);
        {
            let mut lock = BOUNCE_BUFFER_ALLOCATOR.lock();
            lock.as_mut().unwrap().deallocate(vaddr);
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
        trace!(
            "share DMA: buffer={:#x?}, paddr={:#x}, direction={:?}",
            buffer,
            paddr,
            direction
        );
        paddr
    }

    unsafe fn unshare(paddr: PhysAddr, mut buffer: NonNull<[u8]>, direction: BufferDirection) {
        trace!(
            "unshare DMA: paddr={:#x}, buffer={:#x?}, direction={:?}",
            paddr,
            buffer,
            direction
        );
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
                .deallocate(bounce_buffer_ptr.as_non_null_ptr());
        }
    }
}

fn virt_to_phys(vaddr: usize) -> PhysAddr {
    usize::try_from(isize::try_from(vaddr).unwrap() + DMA_VADDR_TO_PADDR_OFFSET.lock().unwrap())
        .unwrap()
}

fn phys_to_virt(paddr: PhysAddr) -> usize {
    usize::try_from(isize::try_from(paddr).unwrap() - DMA_VADDR_TO_PADDR_OFFSET.lock().unwrap())
        .unwrap()
}

static DMA_VADDR_TO_PADDR_OFFSET: GenericMutex<PanickingMutexSyncOps, Option<isize>> =
    GenericMutex::new(PanickingMutexSyncOps::new(), None);

static BOUNCE_BUFFER_ALLOCATOR: GenericMutex<
    PanickingMutexSyncOps,
    Option<BounceBufferAllocator<Bump>>,
> = GenericMutex::new(PanickingMutexSyncOps::new(), None);
