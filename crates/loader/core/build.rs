#![feature(drain_filter)]
#![feature(int_roundings)]

use std::env;
use std::fs;
use std::ops::Range;
use std::path::PathBuf;

use quote::format_ident;

use loader_embed_aarch64_translation_tables::{MkLeafFnParams, Region, Regions};
use sel4_build_env::SEL4_INCLUDE_DIRS;
use sel4_platform_info::PLATFORM_INFO;
use sel4_rustfmt_helper::Rustfmt;

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

    {
        let out_dir = env::var("OUT_DIR").unwrap();
        let out_path = PathBuf::from(&out_dir).join("translation_tables.rs");
        fs::write(&out_path, mk_translation_tables()).unwrap();
        Rustfmt::detect().format(&out_path);
    }
}

fn mk_translation_tables() -> String {
    let normal_shareability = if sel4_config::sel4_cfg_usize!(MAX_NUM_NODES) > 1 {
        3
    } else {
        0
    };

    let mk_normal_entry = move |params: MkLeafFnParams| {
        params
            .mk_identity()
            .set_access_flag(true)
            .set_attribute_index(4) // select MT_NORMAL
            .set_shareability(normal_shareability)
    };

    let mk_device_entry = |params: MkLeafFnParams| {
        params
            .mk_identity()
            .set_access_flag(true)
            .set_attribute_index(0) // select MT_DEVICE_nGnRnE
    };

    let mut regions = Regions::new();
    for range in PLATFORM_INFO.memory.iter() {
        let range = range.start.try_into().unwrap()..range.end.try_into().unwrap();
        regions = regions.insert(Region::valid(range, mk_normal_entry));
    }
    for range in get_device_regions() {
        regions = regions.insert(Region::valid(range, mk_device_entry));
    }

    let toks = regions.construct_and_embed_table(format_ident!("loader_level_0_table"));
    format!("{}", toks)
}

// HACK
fn get_device_regions() -> Vec<Range<u64>> {
    let page = |start| start..start + 4096;
    sel4_config::sel4_cfg_if! {
        if #[cfg(PLAT_QEMU_ARM_VIRT)] {
            vec![page(0x0900_0000)]
        } else if #[cfg(PLAT_BCM2711)] {
            vec![page(0xfe21_5000)]
        } else {
            compile_error!("Unsupported platform");
        }
    }
}
