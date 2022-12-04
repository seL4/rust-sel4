use std::env;
use std::fs;
use std::path::PathBuf;

use sel4_config_data::get_kernel_config;
use sel4_config_generic_build_core::ConfigurationExt;
use sel4_rustfmt_helper::Rustfmt;

fn main() {
    let config = get_kernel_config();
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir).join("gen.rs");
    fs::write(&out_path, format!("{}", config.generate_consts_fragment())).unwrap();
    Rustfmt::detect().format(&out_path);
}
