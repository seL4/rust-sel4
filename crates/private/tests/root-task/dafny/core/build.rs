//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env;
use std::fs;
use std::path::PathBuf;

use proc_macro2::TokenStream;
use quote::quote;

// Note
// The following doesn't work because `include!()`ed contents can't start with inner attributes:
//
// ```
// mod translated {
//     include!(env!(ORIGINAL_TRANSLATED_MOD_PATH));
// }
// ```
//
// Also, using the `sel4-mod-in-out-dir` crate requires `#[feature(proc_macro_hygiene)]` (for
// now).

const TRANSLATED_IN_ENV: &str = "TRANSLATED";

const TRANSLATED_OUT_ENV: &str = "TRANSLATED_OUT";

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let path_in = env::var(TRANSLATED_IN_ENV).unwrap();
    let path_out = PathBuf::from(&out_dir).join("translated.rs");

    let toks_in = fs::read_to_string(&path_in)
        .unwrap()
        .parse::<TokenStream>()
        .unwrap();

    // Work around the fact that `include!()`ed contents can't start with inner attributes
    let toks_out = quote! {
        pub use indirect::*;
        mod indirect {
            #toks_in
        }
    };

    fs::write(&path_out, format!("{toks_out}")).unwrap();

    println!("cargo::rerun-if-env-changed={TRANSLATED_IN_ENV}");
    println!("cargo::rerun-if-changed={path_in}");
    println!(
        "cargo::rustc-env={TRANSLATED_OUT_ENV}={}",
        path_out.display()
    );
}
