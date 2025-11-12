//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::cell::RefCell;

use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};

#[cfg(not(target_thread_local))]
compile_error!("");

#[thread_local]
static RNG: RefCell<Option<SmallRng>> = RefCell::new(None);

const DEFAULT_SEED: u64 = 0;

pub fn seed_dummy_custom_getrandom(seed: u64) {
    assert!(RNG.replace(Some(SmallRng::seed_from_u64(seed))).is_none());
}

fn dummy_custom_getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    if RNG.borrow().is_none() {
        seed_dummy_custom_getrandom(DEFAULT_SEED);
    }
    RNG.borrow_mut().as_mut().unwrap().fill_bytes(buf);
    Ok(())
}

getrandom::register_custom_getrandom!(dummy_custom_getrandom);
