#![no_std]
#![no_main]

use sel4_root_task::{debug_println, root_task};

#[root_task]
fn main(_: &sel4::BootInfo) -> ! {
    // TODO
    debug_println!("TEST_PASS");

    sel4::BootInfo::init_thread_tcb().tcb_suspend().unwrap();
    unreachable!()
}
