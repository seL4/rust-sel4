#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(exclusive_wrapper)]
#![feature(ptr_to_from_bits)]

mod rt;

fn main(bootinfo: &sel4::BootInfo) -> ! {
    sel4::debug_println!("Hello, World!");
    sel4::BootInfo::init_thread_tcb().suspend().unwrap();
    unreachable!()
}
