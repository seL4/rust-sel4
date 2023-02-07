fn main() {
    // Relax p_align requirements to reduce dead space in later rendered ELF (default is 64K)
    println!("cargo:rustc-link-arg=-z");
    println!("cargo:rustc-link-arg=max-page-size=4096");
}
