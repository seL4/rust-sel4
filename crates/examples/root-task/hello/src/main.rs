#![no_std]
#![no_main]
#![feature(never_type)]

use sel4_root_task::root_task;

#[root_task]
fn main(_bootinfo: &sel4::BootInfo) -> sel4::Result<!> {
    sel4::debug_println!("Hello, World!");

    sel4::BootInfo::init_thread_tcb().tcb_suspend()?;

    unreachable!()
}
