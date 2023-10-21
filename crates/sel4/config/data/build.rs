//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env;
use std::fs::File;
use std::path::PathBuf;

use sel4_build_env::find_in_libsel4_include_dirs;
use sel4_config_generic_types::Configuration;

fn main() {
    let config_json_path = find_in_libsel4_include_dirs("kernel/gen_config.json");
    let config =
        Configuration::new(serde_json::from_reader(File::open(config_json_path).unwrap()).unwrap());
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir).join("kernel_config.json");
    serde_json::to_writer_pretty(File::create(out_path).unwrap(), &config).unwrap()
}
