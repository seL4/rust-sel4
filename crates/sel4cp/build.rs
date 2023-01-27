use std::env;
use std::fs;
use std::path::PathBuf;

const DEFAULT_STACK_SIZE: usize = 4096 * 4;
const DEFAULT_HEAP_SIZE: usize = 0;

const STACK_SIZE_ENV: &str = "SEL4CP_PD_STACK_SIZE";
const HEAP_SIZE_ENV: &str = "SEL4_RUNTIME_ROOT_TASK_HEAP_SIZE";

fn main() {
    let stack_size = env::var(STACK_SIZE_ENV)
        .map(|v| v.parse().unwrap())
        .unwrap_or(DEFAULT_STACK_SIZE);

    let heap_size = env::var(HEAP_SIZE_ENV)
        .map(|v| v.parse().unwrap())
        .unwrap_or(DEFAULT_HEAP_SIZE);

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(
        out_dir.join("stack_size.fragment.rs"),
        format!("{}", stack_size),
    )
    .unwrap();

    fs::write(
        out_dir.join("heap_size.fragment.rs"),
        format!("{}", heap_size),
    )
    .unwrap();

    println!("cargo:rerun-if-env-changed={}", STACK_SIZE_ENV);
    println!("cargo:rerun-if-env-changed={}", HEAP_SIZE_ENV);
}
