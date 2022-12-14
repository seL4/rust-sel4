#![feature(int_roundings)]

use std::env;
use std::fs;
use std::path::PathBuf;

use quote::format_ident;

use loader_embed_aarch64_translation_tables::{MkLeafFnParams, Region, Regions};
use sel4_rustfmt_helper::Rustfmt;

fn main() {
    let payload = loader_payload_at_build_time::get();
    let loader_phys_start = payload.info.kernel_image.phys_addr_range.end;

    let out_dir = env::var("OUT_DIR").unwrap();

    {
        let out_path = PathBuf::from(&out_dir).join("loader_phys_start.fragment.rs");
        fs::write(&out_path, format!("{}", loader_phys_start)).unwrap();
    }

    {
        let out_path = PathBuf::from(&out_dir).join("translation_tables.rs");
        fs::write(&out_path, mk_translation_tables()).unwrap();
        Rustfmt::detect().format(&out_path);
    }

    println!("cargo:rustc-link-arg=-Ttext=0x{:x}", loader_phys_start);

    // println!("cargo:rustc-link-arg=--verbose");
    // println!("cargo:rustc-env=RUSTC_LOG=rustc_codegen_ssa::back::link=info");
}

fn mk_translation_tables() -> String {
    let payload = loader_payload_at_build_time::get();
    let info = &payload.info.kernel_image;
    let phys_to_virt_offset = u64::try_from(info.phys_to_virt_offset).unwrap();
    let virt_start = info.phys_addr_range.start + phys_to_virt_offset;
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
