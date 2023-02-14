#![no_std]
#![no_main]

use sel4_root_task_runtime::{debug_println, main};

#[main]
fn main(_: &sel4::BootInfo) -> ! {
    // TODO
    debug_println!("TEST_PASS");

    sel4::BootInfo::init_thread_tcb().tcb_suspend().unwrap();
    unreachable!()
}
