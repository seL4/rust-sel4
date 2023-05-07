use std::env;
use std::fs;
use std::path::PathBuf;

use quote::quote;

use sel4_config_generic_build_core::{Configuration, ConfigurationExt};
use sel4_rustfmt_helper::Rustfmt;

const SEL4_LOADER_CONFIG_ENV: &str = "SEL4_LOADER_CONFIG";

fn main() {
    let config_path = env::var(SEL4_LOADER_CONFIG_ENV)
        .unwrap_or_else(|_| panic!("{} must be set", SEL4_LOADER_CONFIG_ENV));
    let config = Configuration::read_json(&config_path).unwrap();
    let fragment = config.generate_data_fragment(quote! { mk }, quote! { crate::helpers });
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir).join("gen.rs");
    fs::write(&out_path, format!("{fragment}")).unwrap();
    Rustfmt::detect().format(&out_path);

    println!("cargo:rerun-if-env-changed={}", SEL4_LOADER_CONFIG_ENV);
    println!("cargo:rerun-if-changed={}", config_path);
}
