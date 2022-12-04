use sel4_build_env::SEL4_INCLUDE_DIRS;

fn main() {
    let asm_files = glob::glob("asm/aarch64/*.S")
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    cc::Build::new()
        .files(&asm_files)
        .target("aarch64-unknown-none")
        .includes(SEL4_INCLUDE_DIRS.get().iter())
        .compile("asm");

    for path in &asm_files {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}
