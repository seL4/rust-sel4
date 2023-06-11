use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;
use core::ops::Range;
use core::ptr::{self, NonNull};

use dlmalloc::{Allocator as DlmallocAllocator, Dlmalloc};
use log::{debug, info, trace, warn};
use virtio_drivers::{BufferDirection, Hal, PhysAddr, PAGE_SIZE};

use sel4_sync::{GenericMutex, PanickingMutexSyncOps};

pub(crate) struct HalImpl;

impl HalImpl {
    pub(crate) fn init(dma_vaddr_range: Range<*mut u8>, dma_vaddr_to_paddr_offset: isize) {
        {
            let mut lock = DMA_VADDR_TO_PADDR_OFFSET.lock();
            *lock = Some(dma_vaddr_to_paddr_offset);
        }

        {
            let mut lock = DMA_ALLOCATOR.lock();
            *lock = Some(Dlmalloc::new_with_allocator(DmaAllocator {
                free: RefCell::new(dma_vaddr_range),
            }));
        }
    }
}

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let layout = Layout::from_size_align(pages * PAGE_SIZE, PAGE_SIZE).unwrap();
        // Safe because the layout has a non-zero size.
        let vaddr = {
            let mut lock = DMA_ALLOCATOR.lock();
            let allocator = lock.as_mut().unwrap();
            unsafe { allocator.calloc(layout.size(), layout.align()) }
        };
        let vaddr = if let Some(vaddr) = NonNull::new(vaddr) {
            vaddr
        } else {
            panic!("layout: {:?}", layout)
        };
        let paddr = virt_to_phys(vaddr.as_ptr() as _);
        trace!(
            "alloc DMA: paddr={:#x}, vaddr={:#x?}, pages={}",
            paddr,
            vaddr,
            pages
        );
        (paddr, vaddr)
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, vaddr: NonNull<u8>, pages: usize) -> i32 {
        trace!("dealloc DMA: paddr={:#x}, pages={}", paddr, pages);
        let layout = Layout::from_size_align(pages * PAGE_SIZE, PAGE_SIZE).unwrap();
        // Safe because the memory was allocated by `dma_alloc` above using the same allocator, and
        // the layout is the same as was used then.
        {
            let mut lock = DMA_ALLOCATOR.lock();
            let allocator = lock.as_mut().unwrap();
            allocator.free(vaddr.as_ptr(), layout.size(), layout.align());
        }
        0
    }

    unsafe fn mmio_phys_to_virt(_paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        panic!()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        let paddr = virt_to_phys(vaddr);
        trace!("share DMA: buffer={:#x?}, paddr={:#x}", buffer, paddr);
        paddr
    }

    unsafe fn unshare(paddr: PhysAddr, buffer: NonNull<[u8]>, _direction: BufferDirection) {
        trace!("unshare DMA: paddr={:#x}, buffer={:#x?}", paddr, buffer);
        // trace!("unshare DMA: buffer value={:x?}", buffer.as_ref());
    }
}

fn virt_to_phys(vaddr: usize) -> PhysAddr {
    usize::try_from(isize::try_from(vaddr).unwrap() + DMA_VADDR_TO_PADDR_OFFSET.lock().unwrap())
        .unwrap()
}

static DMA_VADDR_TO_PADDR_OFFSET: GenericMutex<PanickingMutexSyncOps, Option<isize>> =
    GenericMutex::new(PanickingMutexSyncOps::new(), None);

static DMA_ALLOCATOR: GenericMutex<PanickingMutexSyncOps, Option<Dlmalloc<DmaAllocator>>> =
    GenericMutex::new(PanickingMutexSyncOps::new(), None);

struct DmaAllocator {
    free: RefCell<Range<*mut u8>>,
}

unsafe impl Send for DmaAllocator {}

unsafe impl DlmallocAllocator for DmaAllocator {
    fn alloc(&self, size: usize) -> (*mut u8, usize, u32) {
        let mut free = self.free.borrow_mut();
        let start = free.start;
        let end = start.wrapping_offset(size.try_into().unwrap());
        if end > free.end {
            (ptr::null_mut(), 0, 0)
        } else {
            free.start = end;
            (start, size, 0)
        }
    }

    fn remap(&self, _ptr: *mut u8, _oldsize: usize, _newsize: usize, _can_move: bool) -> *mut u8 {
        ptr::null_mut()
    }

    fn free_part(&self, _ptr: *mut u8, _oldsize: usize, _newsize: usize) -> bool {
        false
    }

    fn free(&self, _ptr: *mut u8, _size: usize) -> bool {
        false
    }

    fn can_release_part(&self, _flags: u32) -> bool {
        false
    }

    fn allocates_zeros(&self) -> bool {
        false
    }

    fn page_size(&self) -> usize {
        // TODO
        4096
    }
}
