//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

extern crate alloc;

use alloc::sync::Arc;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use sel4_simple_task_application_config_types::*;
use sel4_simple_task_runtime::{debug_println, main_json};
use sel4_simple_task_threading::StaticThread;
use sel4_sync::{RawNotificationMutex, lock_api::Mutex};

sel4_test_capdl::embed_capdl_script!("../cdl.py");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub lock_nfn: ConfigCPtr<sel4::cap::Notification>,
    pub barrier_nfn: ConfigCPtr<sel4::cap::Notification>,
    pub secondary_thread: ConfigCPtr<StaticThread>,
    pub foo: Vec<i32>,
}

const INITIAL_VALUE: i32 = 0;

#[main_json]
fn main(config: Config) {
    debug_println!("{:#?}", config);

    let lock = Arc::new(Mutex::from_raw(
        RawNotificationMutex::new(config.lock_nfn.get()),
        INITIAL_VALUE,
    ));
    let barrier_nfn = config.barrier_nfn.get();

    config.secondary_thread.get().start({
        let lock = lock.clone();
        move || {
            {
                let mut value = lock.lock();
                *value += 1;
            }
            debug_println!("secondary thread");
            barrier_nfn.signal();
        }
    });

    {
        let mut value = lock.lock();
        *value += 1;
    }

    barrier_nfn.wait();

    {
        let value = lock.lock();
        assert_eq!(*value, 2);
    }

    debug_println!("primary thread");

    sel4_test_capdl::indicate_success()
}
