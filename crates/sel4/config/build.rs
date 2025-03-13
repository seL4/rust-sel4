//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env;
use std::fs;
use std::path::PathBuf;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use sel4_config_data::get_kernel_config;
use sel4_config_types::{Configuration, Value};

fn main() {
    let toks = generate_consts(get_kernel_config());
    let formatted = prettyplease::unparse(&syn::parse2(toks).unwrap());
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir).join("consts_gen.rs");
    fs::write(out_path, formatted).unwrap();
}

pub fn generate_consts(config: &Configuration) -> TokenStream {
    let items = config.iter().map(|(k, v)| {
        let k = format_ident!("{}", k);
        let tv = match v {
            Value::Bool(v) => {
                quote! {
                    bool = #v
                }
            }
            Value::String(v) => {
                quote! {
                    &str = #v
                }
            }
        };
        quote! {
            pub const #k: #tv;
        }
    });

    quote! {
        #(#items)*
    }
}
