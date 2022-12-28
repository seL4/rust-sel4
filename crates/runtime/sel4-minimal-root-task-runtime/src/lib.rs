#![no_std]
#![feature(core_intrinsics)]

extern crate sel4_runtime_building_blocks_root_task_head;

use sel4_runtime_building_blocks_termination::Termination;

pub use sel4_minimal_root_task_runtime_macros::main;

#[macro_export]
macro_rules! declare_main {
    ($main:path) => {
        #[no_mangle]
        pub extern "C" fn __rust_entry(bootinfo: *const $crate::_private::seL4_BootInfo) -> ! {
            $crate::_private::run_main($main, bootinfo)
        }
    };
}

pub fn run_main<T: Termination>(
    f: impl Fn(&sel4::BootInfo) -> T,
    bootinfo: *const sel4::sys::seL4_BootInfo,
) -> ! {
    let bootinfo = unsafe { sel4::BootInfo::from_ptr(bootinfo) };

    #[cfg(feature = "state")]
    unsafe {
        sel4::set_ipc_buffer(bootinfo.ipc_buffer());
    }

    f(&bootinfo).report(&mut sel4::DebugWrite).unwrap();

    let r = sel4::BootInfo::init_thread_tcb()
        .with(&mut unsafe { bootinfo.ipc_buffer() })
        .tcb_suspend()
        .unwrap();
    sel4::debug_println!("Failed to suspend: {:?}", r);
    abort()
}

#[cfg(feature = "panic-handler")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    sel4::debug_println!("{}", info);
    abort()
}

fn abort() -> ! {
    sel4::debug_println!("(aborting)");
    core::intrinsics::abort()
}

#[doc(hidden)]
pub mod _private {
    pub use crate::run_main;
    pub use sel4::sys::seL4_BootInfo;
}
