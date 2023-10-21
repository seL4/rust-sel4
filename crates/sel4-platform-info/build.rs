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
use serde::{Deserialize, Serialize};

use sel4_build_env::get_with_sel4_prefix_relative_fallback;

const SEL4_PLATFORM_INFO_ENV: &str = "SEL4_PLATFORM_INFO";

#[sel4_config::sel4_cfg(WORD_SIZE = "32")]
type Word = u32;

#[sel4_config::sel4_cfg(WORD_SIZE = "64")]
type Word = u64;

fn main() {
    let platform_info_path =
        get_with_sel4_prefix_relative_fallback(SEL4_PLATFORM_INFO_ENV, "support/platform_gen.yaml");
    let platform_info: PlatformInfoForBuildSystem =
        serde_yaml::from_reader(fs::File::open(platform_info_path).unwrap()).unwrap();
    let fragment = platform_info.embed();
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir).join("gen.rs");
    fs::write(out_path, format!("{fragment}")).unwrap();
}

type Ranges = Vec<Range<Word>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlatformInfoForBuildSystem {
    memory: Ranges,
    devices: Ranges,
}

impl PlatformInfoForBuildSystem {
    fn embed(&self) -> TokenStream {
        let ty = match sel4_config::sel4_cfg_usize!(WORD_SIZE) {
            32 => quote!(u32),
            64 => quote!(u64),
            _ => unreachable!(),
        };
        let memory = embed_ranges(&self.memory);
        let devices = embed_ranges(&self.devices);
        quote! {
            pub const PLATFORM_INFO: PlatformInfo<#ty> = PlatformInfo {
                memory: #memory,
                devices: #devices,
            };
        }
    }
}

fn embed_ranges(ranges: &Ranges) -> TokenStream {
    let toks = format!("{ranges:?}").parse::<TokenStream>().unwrap();
    quote! {
        &#toks
    }
}
