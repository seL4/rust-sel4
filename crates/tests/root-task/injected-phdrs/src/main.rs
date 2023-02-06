#![no_std]
#![no_main]

use sel4_full_root_task_runtime::{debug_println, main};

#[main]
fn main(_: &sel4::BootInfo) -> ! {
    assert_eq!(
        sel4_runtime_phdrs::injected::get_phdrs(),
        sel4_runtime_phdrs::embedded::get_phdrs()
    );
    debug_println!("TEST_PASS");
    sel4::BootInfo::init_thread_tcb().tcb_suspend().unwrap();
    unreachable!()
}
