//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(unused_imports)]

use std::sync::LazyLock;

pub use sel4_config_types::Configuration;

pub fn get_kernel_config() -> &'static Configuration {
    &KERNEL_CONFIG
}

static KERNEL_CONFIG: LazyLock<Configuration> =
    LazyLock::new(|| serde_json::from_str(KERNEL_CONFIG_JSON).unwrap());

const KERNEL_CONFIG_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/kernel_config.json"));
