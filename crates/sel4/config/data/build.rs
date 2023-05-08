use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};

use sel4_build_env::SEL4_INCLUDE_DIRS;
use sel4_config_generic_types::Configuration;

fn main() {
    let config = {
        let kernel_config = from_path(&find("kernel/gen_config.json"));
        let libsel4_config = from_path(&find("sel4/gen_config.json"));
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
