use std::env;
use std::fs;
use std::path::PathBuf;

use quote::format_ident;

use loader_embed_aarch64_translation_tables::{embed, Region, PHYS_BOUNDS};
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
    let regions = vec![
        Region::new(0..virt_start, |vaddr| {
            vaddr
            | (1 << 10) // access flag
            | (0 << 2) // select MT_DEVICE_nGnRnE
            | (1 << 0) // mark as valid
        }),
        Region::new(virt_start..PHYS_BOUNDS.end, move |vaddr| {
            (vaddr - phys_to_virt_offset)
            | (1 << 10) // access flag
            | (4 << 2) // select MT_NORMAL
            | (1 << 0) // mark as valid
            | if sel4_config::sel4_cfg_usize!(MAX_NUM_NODES) > 1 { 3 << 8 } else { 0 }
        }),
    ];
    let toks = embed(format_ident!("boot_level_0_table"), regions.iter());
    format!("{}", toks)
}
