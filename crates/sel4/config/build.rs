//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env;
use std::fs;
use std::path::PathBuf;

use sel4_config_data::get_kernel_config;
use sel4_config_generic::generate_consts;
use sel4_rustfmt_helper::Rustfmt;

fn main() {
    let toks = generate_consts(get_kernel_config());
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir).join("consts_gen.rs");
    fs::write(&out_path, format!("{}", toks)).unwrap();
    Rustfmt::detect().format(&out_path);
}
