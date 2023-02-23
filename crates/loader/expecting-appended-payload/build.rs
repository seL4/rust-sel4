#![feature(int_roundings)]

use std::env;
use std::fs;
use std::path::PathBuf;

use quote::format_ident;

use loader_embed_aarch64_translation_tables::{MkLeafFnParams, Region, Regions};
use sel4_rustfmt_helper::Rustfmt;

fn main() {
    // let loader_phys_start = payload.info.kernel_image.phys_addr_range.end;
    let loader_phys_start = 0;

    let out_dir = env::var("OUT_DIR").unwrap();

    let kernel_phys_start = 0x100000;
    let kernel_phys_to_virt_offset = 0;

    {
        let out_path = PathBuf::from(&out_dir).join("translation_tables.rs");
        fs::write(
            &out_path,
            mk_translation_tables(kernel_phys_start, kernel_phys_to_virt_offset),
        )
        .unwrap();
        Rustfmt::detect().format(&out_path);
    }

    // Note that -Ttext={} is incompatible with --no-rosegment (no error),
    // just bad output. See the "Default program headers" section of:
    // https://maskray.me/blog/2020-12-19-lld-and-gnu-linker-incompatibilities
    println!(
        "cargo:rustc-link-arg=--image-base=0x{:x}",
        loader_phys_start
    );

    println!("cargo:rustc-link-arg=-z");
    println!("cargo:rustc-link-arg=max-page-size=4096");

    // No use in loader.
    // Remove unnecessary alignment gap between segments.
    println!("cargo:rustc-link-arg=--no-rosegment");

    // println!("cargo:rustc-link-arg=--verbose");
    // println!("cargo:rustc-env=RUSTC_LOG=rustc_codegen_ssa::back::link=info");
}

fn mk_translation_tables(kernel_phys_start: u64, kernel_phys_to_virt_offset: u64) -> String {
    let phys_to_virt_offset = kernel_phys_to_virt_offset;
    let virt_start = kernel_phys_start + phys_to_virt_offset;
    let virt_end = virt_start.next_multiple_of(1 << 39);

    let normal_shareability = if sel4_config::sel4_cfg_usize!(MAX_NUM_NODES) > 1 {
        3
    } else {
        0
    };

    let identity_map = |params: MkLeafFnParams| {
        params
            .mk_identity()
            .set_access_flag(true)
            .set_attribute_index(0) // select MT_DEVICE_nGnRnE
    };

    let kernel_map = move |params: MkLeafFnParams| {
        params
            .mk(|vaddr| vaddr - phys_to_virt_offset)
            .set_access_flag(true)
            .set_attribute_index(4) // select MT_NORMAL
            .set_shareability(normal_shareability)
    };

    let regions = Regions::new()
        .insert(Region::valid(0..virt_start, identity_map))
        .insert(Region::valid(virt_start..virt_end, kernel_map));

    let toks = regions.construct_and_embed_table(format_ident!("kernel_boot_level_0_table"));
    format!("{}", toks)
}
