#![no_std]
#![no_main]

use sel4_full_root_task_runtime::{debug_println, main};

#[main]
fn main(_: &sel4::BootInfo) -> ! {
    assert_eq!(
        sel4_runtime_building_blocks_injected_phdrs::get_phdrs(),
        sel4_runtime_building_blocks_embedded_phdrs::get_phdrs()
    );
    debug_println!("TEST_PASS");
    sel4::BootInfo::init_thread_tcb().tcb_suspend().unwrap();
    unreachable!()
}
