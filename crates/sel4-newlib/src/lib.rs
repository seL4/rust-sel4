#![no_std]
#![feature(const_slice_from_raw_parts_mut)]
#![feature(slice_ptr_get)]
#![feature(slice_ptr_len)]
#![feature(sync_unsafe_cell)]

#[allow(unused_imports)]
use core::ffi::{c_char, c_int, c_uint, c_void};

use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;

#[derive(Default)]
pub struct Implementations {
    #[cfg(feature = "_exit")]
    pub _exit: Option<fn()>,
    #[cfg(feature = "_sbrk")]
    pub _sbrk: Option<fn(incr: c_int) -> *mut c_void>,
    #[cfg(feature = "_write")]
    pub _write: Option<fn(file: c_int, ptr: *const c_char, len: c_int)>,
    #[cfg(feature = "__trunctfdf2")]
    pub __trunctfdf2: Option<fn(a: LongDoublePlaceholder) -> f64>,
}

static IMPLEMENTATIONS: ImmediateSyncOnceCell<Implementations> = ImmediateSyncOnceCell::new();

pub fn set_implementations(implementations: Implementations) {
    IMPLEMENTATIONS.set(implementations).unwrap_or_else(|_| {
        panic!("set_implementations() has already been called, or has not yet been completed")
    })
}

pub fn try_get_implementations() -> Option<&'static Implementations> {
    IMPLEMENTATIONS.get()
}

pub fn get_implementations() -> &'static Implementations {
    try_get_implementations()
        .unwrap_or_else(|| panic!("set_implementations() has not yet been called"))
}

#[allow(unused_macros)]
macro_rules! get_impl {
    ($symbol:ident) => {
        crate::get_implementations()
            .$symbol
            .unwrap_or_else(|| unimplemented!(stringify!($symbol)))
    };
}

#[cfg(feature = "_exit")]
mod impl_exit {
    #[no_mangle]
    extern "C" fn _exit() {
        get_impl!(_exit)()
    }
}

#[cfg(feature = "_sbrk")]
pub use impl_sbrk::*;

#[cfg(feature = "_sbrk")]
mod impl_sbrk {
    use super::*;

    use core::cell::SyncUnsafeCell;
    use core::ptr;
    use core::sync::atomic::{AtomicIsize, Ordering};

    use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;

    pub struct StaticHeap<const N: usize>(SyncUnsafeCell<[u8; N]>);

    impl<const N: usize> StaticHeap<N> {
        pub const fn new() -> Self {
            Self(SyncUnsafeCell::new([0; N]))
        }

        const fn bounds(&self) -> *mut [u8] {
            ptr::slice_from_raw_parts_mut(self.0.get().cast(), N)
        }
    }

    struct StaticHeapState {
        watermark: AtomicIsize,
        ptr: *mut [u8],
    }

    unsafe impl Sync for StaticHeapState {}

    impl StaticHeapState {
        const fn new<const N: usize>(heap: &StaticHeap<N>) -> Self {
            Self {
                watermark: AtomicIsize::new(0),
                ptr: heap.bounds(),
            }
        }

        fn sbrk(&self, incr: isize) -> *mut u8 {
            let old = self.watermark.fetch_add(incr, Ordering::SeqCst);
            let new = old + incr;
            assert!(new >= 0);
            assert!(new <= self.ptr.len().try_into().unwrap());
            unsafe { self.ptr.as_mut_ptr().offset(old).cast() }
        }
    }

    static STATIC_HEAP_STATE: ImmediateSyncOnceCell<StaticHeapState> = ImmediateSyncOnceCell::new();

    pub fn sbrk_with_static_heap(incr: c_int) -> *mut c_void {
        STATIC_HEAP_STATE
            .get()
            .expect(
                "set_static_heap_for_sbrk() has not yet been called, or has not yet been completed",
            )
            .sbrk(incr.try_into().unwrap())
            .cast()
    }

    pub fn set_static_heap_for_sbrk<const N: usize>(static_heap: &StaticHeap<N>) {
        STATIC_HEAP_STATE
            .set(StaticHeapState::new(static_heap))
            .ok()
            .expect("set_static_heap_for_sbrk() has already been called")
    }

    #[no_mangle]
    extern "C" fn _sbrk(incr: c_int) -> *mut c_void {
        get_impl!(_sbrk)(incr)
    }
}

#[cfg(feature = "_write")]
pub use impl_write::*;

#[cfg(feature = "_write")]
mod impl_write {
    use super::*;

    #[no_mangle]
    extern "C" fn _write(file: c_int, ptr: *const c_char, len: c_int) {
        get_impl!(_write)(file, ptr, len)
    }

    #[cfg(feature = "sel4-panicking-env")]
    pub use with_sel4_panicking_env::*;

    #[cfg(feature = "sel4-panicking-env")]
    mod with_sel4_panicking_env {
        use super::*;

        use core::slice;

        use sel4_panicking_env::debug_put_char;

        pub fn write_with_debug_put_char(_file: c_int, ptr: *const c_char, len: c_int) {
            let bytes = unsafe { slice::from_raw_parts(ptr.cast::<u8>(), len.try_into().unwrap()) };
            for b in bytes {
                debug_put_char(*b);
            }
        }
    }
}

#[cfg(feature = "__trunctfdf2")]
pub use impl__trunctfdf2::*;

#[cfg(feature = "__trunctfdf2")]
#[allow(non_snake_case)]
mod impl__trunctfdf2 {
    #[repr(C, align(16))]
    pub struct LongDoublePlaceholder(pub [u8; 16]);

    #[no_mangle]
    extern "C" fn __trunctfdf2(a: LongDoublePlaceholder) -> f64 {
        get_impl!(__trunctfdf2)(a)
    }
}

extern "C" {
    #[link_name = "srand"]
    fn newlib_srand(seed: c_uint);
}

pub fn srand(seed: c_uint) {
    unsafe {
        newlib_srand(seed);
    }
}
