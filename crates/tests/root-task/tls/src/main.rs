#![no_std]
#![no_main]
#![feature(thread_local)]

use sel4_root_task_runtime::{debug_println, main};

#[repr(C, align(8192))]
struct Y(i32);

#[no_mangle]
#[thread_local]
static X: Y = Y(1337);

#[main]
fn main(_: &sel4::BootInfo) -> ! {
    debug_println!("{}", X.0);
    assert_eq!(X.0, 1337);
    debug_println!("TEST_PASS");
    sel4::BootInfo::init_thread_tcb().tcb_suspend().unwrap();
    unreachable!()
}
