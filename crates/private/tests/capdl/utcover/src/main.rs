//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

extern crate alloc;

use serde::{Deserialize, Serialize};

use sel4_simple_task_config_types::*;
use sel4_simple_task_runtime::{debug_println, main_json};

sel4_test_capdl::embed_capdl_script!("../cdl.py");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub frame: ConfigCPtr<sel4::cap::Granule>,
}

#[main_json]
fn main(config: Config) {
    debug_println!("{:#?}", config);

    debug_println!(
        "addr: {:#x}",
        config.frame.get().frame_get_address().unwrap()
    );

    sel4_test_capdl::indicate_success()
}
