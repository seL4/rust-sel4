const HEAP_SIZE_ENV: &str = "SEL4_RUNTIME_ROOT_TASK_HEAP_SIZE";

fn main() {
    println!("cargo:rerun-if-env-changed={}", HEAP_SIZE_ENV);
}
