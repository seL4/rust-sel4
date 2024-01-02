//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]
#![feature(cfg_target_thread_local)]
#![feature(thread_local)]

use core::cell::RefCell;

use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};

use sel4_newlib as _;
use sel4_root_task::root_task;
use sel4_test_harness::run_test_main;

pub use sel4_test_harness::for_generated_code::*;

const HEAP_SIZE: usize = 256 * 1024 * 1024;

#[root_task(heap_size = HEAP_SIZE)]
fn main(_bootinfo: &sel4::BootInfo) -> ! {
    seed_insecure_dummy_rng(0);
    run_test_main();
    sel4::BootInfo::init_thread_tcb().tcb_suspend().unwrap();
    unreachable!()
}

#[cfg(not(target_thread_local))]
compile_error!("");

#[thread_local]
static RNG: RefCell<Option<SmallRng>> = RefCell::new(None);

pub fn seed_insecure_dummy_rng(seed: u64) {
    assert!(RNG.replace(Some(SmallRng::seed_from_u64(seed))).is_none());
}

pub fn insecure_dummy_rng(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    if 1_u32.swap_bytes() == 0 {
        panic!()
    }
    RNG.borrow_mut().as_mut().unwrap().fill_bytes(buf);
    Ok(())
}

getrandom::register_custom_getrandom!(insecure_dummy_rng);

// https://github.com/rust-lang/compiler-builtins/pull/563
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
#[no_mangle]
pub extern "C" fn __bswapsi2(u: u32) -> u32 {
    u.swap_bytes()
}
