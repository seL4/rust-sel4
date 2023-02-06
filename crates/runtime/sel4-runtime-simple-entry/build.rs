const STACK_SIZE_ENV: &str = "SEL4_RUNTIME_STACK_SIZE";

fn main() {
    println!("cargo:rerun-if-env-changed={}", STACK_SIZE_ENV);
}
