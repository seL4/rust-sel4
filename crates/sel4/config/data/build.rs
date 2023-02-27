use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use quote::quote;

use sel4_build_env::SEL4_INCLUDE_DIRS;
use sel4_config_generic_build_core::{Configuration, ConfigurationExt};
use sel4_rustfmt_helper::Rustfmt;

fn main() {
    let config = {
        let kernel_config = Configuration::read_json(&find("kernel/gen_config.json")).unwrap();
        let libsel4_config = Configuration::read_json(&find("sel4/gen_config.json")).unwrap();
        let mut this = Configuration::empty();
        this.append(kernel_config);
        this.append(libsel4_config);
        this
    };

    let fragment = config.generate_data_fragment(quote! { mk }, quote! { crate::helpers });
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir).join("gen.rs");
    fs::write(&out_path, format!("{fragment}")).unwrap();
    Rustfmt::detect().format(&out_path);
}

fn find(relative_path: impl AsRef<Path>) -> PathBuf {
    for d in SEL4_INCLUDE_DIRS.get().iter() {
        let path = Path::new(d).join(relative_path.as_ref());
        if path.exists() {
            println!("cargo:rerun-if-changed={}", path.display());
            return path;
        }
    }
    panic!()
}
