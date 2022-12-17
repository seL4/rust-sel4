#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(exclusive_wrapper)]
#![feature(ptr_to_from_bits)]

mod fmt;
mod rt;

use fmt::debug_print;

fn main(bootinfo: &sel4_sys::seL4_BootInfo) -> ! {
    let ipc_buffer = unsafe { &mut *bootinfo.ipcBuffer };
    debug_print!("Hello, World!\n");
    let _ = ipc_buffer.seL4_TCB_Suspend(sel4_sys::seL4_RootCapSlot::seL4_CapInitThreadTCB.into());
    unreachable!()
}
