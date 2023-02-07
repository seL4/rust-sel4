use std::env;
use std::fs;
use std::path::PathBuf;

use quote::quote;

use loader_build_env::SEL4_LOADER_CONFIG;
use sel4_config_generic_build_core::{Configuration, ConfigurationExt};
use sel4_rustfmt_helper::Rustfmt;

fn main() {
    let config_path = SEL4_LOADER_CONFIG.get();
    let config = Configuration::read_json(&config_path).unwrap();
    let fragment = config.generate_data_fragment(quote! { mk }, quote! { crate::helpers });
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir).join("gen.rs");
    fs::write(&out_path, format!("{fragment}")).unwrap();
    Rustfmt::detect().format(&out_path);

    println!("cargo:rerun-if-changed={}", config_path.display());
}
