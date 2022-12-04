use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let payload = loader_payload_at_build_time::get();
    let loader_phys_start = payload.info.kernel_image.phys_addr_range.end;

    // HACK
    {
        let out_dir = env::var("OUT_DIR").unwrap();
        let out_path = PathBuf::from(&out_dir).join("loader_phys_start.fragment.rs");
        fs::write(&out_path, format!("{}", loader_phys_start)).unwrap();
    }

    println!("cargo:rustc-link-arg=-Ttext=0x{:x}", loader_phys_start);

    // println!("cargo:rustc-link-arg=--verbose");
    // println!("cargo:rustc-env=RUSTC_LOG=rustc_codegen_ssa::back::link=info");
}
