use std::env;
use std::fs;
use std::path::PathBuf;

const DEFAULT_HEAP_SIZE: usize = 0;

const HEAP_SIZE_ENV: &str = "SEL4_RUNTIME_ROOT_TASK_HEAP_SIZE";

fn main() {
    let heap_size = env::var(HEAP_SIZE_ENV)
        .map(|v| v.parse().unwrap())
        .unwrap_or(DEFAULT_HEAP_SIZE);

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(
        out_dir.join("heap_size.fragment.rs"),
        format!("{}", heap_size),
    )
    .unwrap();

    println!("cargo:rerun-if-env-changed={}", HEAP_SIZE_ENV);
}

// NOTE
// The following alternative doesn't work without an auxiliary proc-macro
// crate until 'env!' supports types other than &'static str.

// const HEAP_SIZE_ENV_FOR_CODE: &str = "_HEAP_SIZE";

// fn main() {
//     ...
//     println!("cargo:rustc-env={}={}", HEAP_SIZE_ENV_FOR_CODE, heap_size);
//     ...
// }
