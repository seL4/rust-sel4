#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(exclusive_wrapper)]

use core::panic::PanicInfo;

mod fmt;
mod head;

use fmt::debug_print;

#[no_mangle]
extern "C" fn main(bootinfo: *const sel4_sys::seL4_BootInfo) -> ! {
    let bootinfo = unsafe { &*bootinfo };
    let ipc_buffer = unsafe { &mut *bootinfo.ipcBuffer };
    debug_print!("Hello, World!\n");
    let _ = ipc_buffer.seL4_TCB_Suspend(sel4_sys::seL4_RootCapSlot::seL4_CapInitThreadTCB.into());
    unreachable!()
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    debug_print!("{}\n", info);
    core::intrinsics::abort()
}
