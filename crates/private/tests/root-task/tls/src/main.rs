#![no_std]
#![no_main]
#![feature(thread_local)]

use sel4_root_task::{debug_println, root_task};

#[repr(C, align(4096))]
struct Y(i32);

#[no_mangle]
#[thread_local]
static X: Y = Y(1337);

#[root_task]
fn main(_: &sel4::BootInfo) -> ! {
    debug_println!("{}", X.0);
    assert_eq!(X.0, 1337);
    debug_println!("TEST_PASS");
    sel4::BootInfo::init_thread_tcb().tcb_suspend().unwrap();
    unreachable!()
}
