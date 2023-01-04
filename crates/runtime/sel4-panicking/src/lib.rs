#![no_std]
#![feature(lang_items)]
#![feature(never_type)]
#![feature(panic_info_message)]
#![feature(core_intrinsics)]
#![feature(thread_local)]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::any::Any;
use core::cell::Cell;
use core::mem::ManuallyDrop;
use core::panic::Location;
use core::panic::PanicInfo;

use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;
use sel4_runtime_building_blocks_abort::{abort, debug_println};

mod strategy;
mod whether_alloc;

use strategy::{panic_cleanup, start_panic};
pub use whether_alloc::Payload;

#[cfg(not(feature = "alloc"))]
pub use whether_alloc::Region;

#[cfg_attr(not(panic = "unwind"), allow(dead_code))]
pub(crate) fn drop_panic() -> ! {
    debug_println!("Rust panics must be rethrown");
    core::intrinsics::abort()
}

#[cfg_attr(not(panic = "unwind"), allow(dead_code))]
pub(crate) fn foreign_exception() -> ! {
    debug_println!("Rust cannot catch foreign exceptions");
    core::intrinsics::abort()
}

pub fn catch_unwind<R, F: FnOnce() -> R>(f: F) -> Result<R, Payload> {
    union Data<F, R> {
        f: ManuallyDrop<F>,
        r: ManuallyDrop<R>,
        p: ManuallyDrop<Payload>,
    }

    let mut data = Data {
        f: ManuallyDrop::new(f),
    };

    let data_ptr = &mut data as *mut _ as *mut u8;
    unsafe {
        return if core::intrinsics::r#try(do_call::<F, R>, data_ptr, do_catch::<F, R>) == 0 {
            Ok(ManuallyDrop::into_inner(data.r))
        } else {
            Err(ManuallyDrop::into_inner(data.p))
        };
    }

    #[inline]
    fn do_call<F: FnOnce() -> R, R>(data: *mut u8) {
        unsafe {
            let data = data as *mut Data<F, R>;
            let data = &mut (*data);
            let f = ManuallyDrop::take(&mut data.f);
            data.r = ManuallyDrop::new(f());
        }
    }

    #[inline]
    fn do_catch<F: FnOnce() -> R, R>(data: *mut u8, exception: *mut u8) {
        unsafe {
            let data = data as *mut Data<F, R>;
            let data = &mut (*data);
            let payload = panic_cleanup(exception);
            panic_caught();
            data.p = ManuallyDrop::new(payload);
        }
    }
}

// TODO consider supporting nested panics
#[thread_local]
static PANIC_COUNT: Cell<usize> = Cell::new(0);

pub(crate) fn panic_caught() {
    PANIC_COUNT.set(0);
}

pub type PanicHook = &'static (dyn Fn() + Send + Sync);

static PANIC_HOOK: ImmediateSyncOnceCell<PanicHook> = ImmediateSyncOnceCell::new();

pub fn set_hook(hook: PanicHook) {
    PANIC_HOOK.set(hook).unwrap_or_else(|_| panic!())
}

fn default_hook() {}

fn get_hook() -> &'static PanicHook {
    const DEFAULT_HOOK: PanicHook = &default_hook;
    PANIC_HOOK.get().unwrap_or(&DEFAULT_HOOK)
}

fn do_panic(payload: Payload) -> ! {
    if PANIC_COUNT.get() >= 1 {
        debug_println!("thread panicked while processing panic. aborting.");
        abort();
    }
    PANIC_COUNT.set(1);
    (get_hook())();
    let code = start_panic(payload);
    debug_println!("failed to initiate panic, error {}", code);
    abort();
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    debug_println!("{}", info);
    do_panic(Payload::empty())
}

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        use alloc::boxed::Box;

        #[track_caller]
        pub fn panic_any<M: 'static + Any + Send>(msg: M) -> ! {
            debug_println!("panicked at {}", Location::caller());
            do_panic(Payload(Some(Box::new(msg))))
        }
    } else {
        #[track_caller]
        pub fn panic_any<M: 'static + Any + Send>(msg: &'static M) -> ! {
            debug_println!("panicked at {}", Location::caller());
            do_panic(Payload(Some(Region::Ref(msg))))
        }

        #[track_caller]
        pub fn panic_val(msg: usize) -> ! {
            debug_println!("panicked at {}", Location::caller());
            do_panic(Payload(Some(Region::Val(msg))))
        }
    }
}
