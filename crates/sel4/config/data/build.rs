use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};

use sel4_build_env::find_in_libsel4_include_dirs;
use sel4_config_generic_types::Configuration;

fn main() {
    let config = {
        let kernel_config = from_path(&find_in_libsel4_include_dirs("kernel/gen_config.json"));
        let libsel4_config = from_path(&find_in_libsel4_include_dirs("sel4/gen_config.json"));
        let mut this = Configuration::empty();
        this.append(kernel_config);
        this.append(libsel4_config);
        this
    };

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir).join("kernel_config.json");
    serde_json::to_writer_pretty(File::create(out_path).unwrap(), &config).unwrap()
}

fn from_path(path: impl AsRef<Path>) -> Configuration {
    serde_json::from_reader(File::open(path).unwrap()).unwrap()
}
