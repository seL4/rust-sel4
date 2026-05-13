//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env;
use std::fs;
use std::ops::Range;
use std::path::PathBuf;

use proc_macro2::TokenStream;
use quote::quote;

use sel4_build_env::get_with_sel4_prefix_relative_fallback;
use sel4_platform_info_types::OwnedPlatformInfo;

const SEL4_PLATFORM_INFO_ENV: &str = "SEL4_PLATFORM_INFO";

fn main() {
    let platform_info_path =
        get_with_sel4_prefix_relative_fallback(SEL4_PLATFORM_INFO_ENV, "support/platform_gen.yaml");
    let platform_info: OwnedPlatformInfo =
        serde_yaml::from_reader(fs::File::open(platform_info_path).unwrap()).unwrap();
    let fragment = embed(&platform_info);
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir).join("gen.rs");
    fs::write(out_path, format!("{fragment}")).unwrap();
}

fn embed(platform_info: &OwnedPlatformInfo) -> TokenStream {
    let ty = match sel4_config::sel4_cfg_usize!(WORD_SIZE) {
        32 => quote!(u32),
        64 => quote!(u64),
        _ => unreachable!(),
    };
    let memory = embed_ranges(&platform_info.memory);
    let devices = embed_ranges(&platform_info.devices);
    quote! {
        pub const PLATFORM_INFO: PlatformInfo<#ty> = PlatformInfo {
            memory: #memory,
            devices: #devices,
        };
    }
}

fn embed_ranges(ranges: &Vec<Range<u64>>) -> TokenStream {
    let toks = format!("{ranges:?}").parse::<TokenStream>().unwrap();
    quote! {
        &#toks
    }
}
