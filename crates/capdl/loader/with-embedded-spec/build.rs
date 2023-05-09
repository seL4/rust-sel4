fn main() {
    capdl_embedded_spec_validate::run(true);

    // No use in root task.
    // Remove unnecessary alignment gap between segments.
    println!("cargo:rustc-link-arg=--no-rosegment");
}
