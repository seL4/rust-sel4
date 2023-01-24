use std::env;
use std::fs;
use std::path::PathBuf;

const DEFAULT_STACK_SIZE: usize = 4096 * 4;

const STACK_SIZE_ENV: &str = "SEL4CP_PD_STACK_SIZE";

fn main() {
    let stack_size = env::var(STACK_SIZE_ENV)
        .map(|v| v.parse().unwrap())
        .unwrap_or(DEFAULT_STACK_SIZE);

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(
        out_dir.join("stack_size.fragment.rs"),
        format!("{}", stack_size),
    )
    .unwrap();

    println!("cargo:rerun-if-env-changed={}", STACK_SIZE_ENV);
}

// NOTE
// The following alternative doesn't work without an auxiliary proc-macro
// crate until 'env!' supports types other than &'static str.

// const STACK_SIZE_ENV_FOR_CODE: &str = "_STACK_SIZE";

// fn main() {
//     ...
//     println!("cargo:rustc-env={}={}", STACK_SIZE_ENV_FOR_CODE, stack_size);
//     ...
// }
