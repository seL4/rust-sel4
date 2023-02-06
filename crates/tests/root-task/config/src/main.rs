#![no_std]
#![no_main]

use sel4_full_root_task_runtime::{debug_println, main};

#[sel4::sel4_cfg(not(KERNEL_STACK_BITS = "0"))]
#[main]
fn main(_: &sel4::BootInfo) -> ! {
    debug_println!(
        "RETYPE_FAN_OUT_LIMIT: {}",
        sel4::sel4_cfg_usize!(RETYPE_FAN_OUT_LIMIT),
    );
    sel4::sel4_cfg_if! {
        if #[cfg(NUM_PRIORITIES = "0")] {
            compile_error!("uh oh");
        } else {
            debug_println!("NUM_PRIORITIES: {}", sel4::sel4_cfg_usize!(NUM_PRIORITIES));
        }
    }
    debug_println!("TEST_PASS");

    sel4::BootInfo::init_thread_tcb().tcb_suspend().unwrap();
    unreachable!()
}
