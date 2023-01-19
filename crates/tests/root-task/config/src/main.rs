#![no_std]
#![no_main]
#![feature(proc_macro_hygiene)]

use sel4_full_root_task_runtime::{debug_println, main};

#[sel4::sel4_cfg(not(KERNEL_STACK_BITS = "0"))]
#[main]
fn main(_: &sel4::BootInfo) -> ! {
    debug_println!(
        "RETYPE_FAN_OUT_LIMIT: {}",
        sel4::sel4_cfg_usize!(RETYPE_FAN_OUT_LIMIT),
    );
    sel4::sel4_cfg_if! {
        if #[cfg(DISABLE_WFI_WFE_TRAPS)] {
            debug_println!("DISABLE_WFI_WFE_TRAPS");
        } else if #[cfg(any(not(DISABLE_WFI_WFE_TRAPS), NUM_PRIORITIES = "0"))] {
            compile_error!("uh oh");
        }
    }
    debug_println!("TEST_PASS");

    sel4::BootInfo::init_thread_tcb().tcb_suspend().unwrap();
    unreachable!()
}
