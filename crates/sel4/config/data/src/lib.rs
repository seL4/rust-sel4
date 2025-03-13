//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(unused_imports)]

pub use sel4_config_types::Configuration;

pub fn get_kernel_config() -> &'static Configuration {
    &KERNEL_CONFIG
}

lazy_static::lazy_static! {
    static ref KERNEL_CONFIG: Configuration = {
        serde_json::from_str(KERNEL_CONFIG_JSON).unwrap()
    };
}

const KERNEL_CONFIG_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/kernel_config.json"));
