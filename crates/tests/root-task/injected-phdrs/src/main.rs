#![no_std]
#![no_main]

use sel4_root_task_runtime::{debug_println, main};
use sel4_runtime_phdrs::{EmbeddedProgramHeaders, InjectedProgramHeaders};

#[main]
fn main(_: &sel4::BootInfo) -> ! {
    assert_eq!(
        EmbeddedProgramHeaders::finder().find_phdrs(),
        InjectedProgramHeaders::finder().find_phdrs(),
    );
    debug_println!("TEST_PASS");
    sel4::BootInfo::init_thread_tcb().tcb_suspend().unwrap();
    unreachable!()
}
