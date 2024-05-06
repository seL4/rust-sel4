//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]
#![feature(cfg_target_thread_local)]
#![feature(thread_local)]

use sel4_newlib as _;
use sel4_root_task::root_task;
use sel4_test_harness::run_test_main;

pub use sel4_test_harness::for_generated_code::*;

const HEAP_SIZE: usize = 256 * 1024 * 1024;

#[root_task(heap_size = HEAP_SIZE)]
fn main(_bootinfo: &sel4::BootInfoPtr) -> ! {
    init();
    run_test_main();
    sel4::init_thread::suspend_self()
}

fn init() {
    dummy_custom_getrandom::seed_dummy_custom_getrandom(0);
}

mod dummy_custom_getrandom {
    use core::cell::RefCell;

    use rand::rngs::SmallRng;
    use rand::{RngCore, SeedableRng};

    #[cfg(not(target_thread_local))]
    compile_error!("");

    #[thread_local]
    static RNG: RefCell<Option<SmallRng>> = RefCell::new(None);

    pub(crate) fn seed_dummy_custom_getrandom(seed: u64) {
        assert!(RNG.replace(Some(SmallRng::seed_from_u64(seed))).is_none());
    }

    fn dummy_custom_getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
        RNG.borrow_mut().as_mut().unwrap().fill_bytes(buf);
        Ok(())
    }

    getrandom::register_custom_getrandom!(dummy_custom_getrandom);
}
