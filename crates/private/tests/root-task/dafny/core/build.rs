//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env;
use std::fs;
use std::path::PathBuf;

use quote::quote;

const TRANSLATED_ENV: &str = "TRANSLATED";

const TRANSLATED_CONTENTS_DST_ENV: &str = "TRANSLATED_CONTENTS_DST";

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    let translated_src = env::var(TRANSLATED_ENV).unwrap();
    let translated_dst = PathBuf::from(&out_dir).join("translated.rs");
    fs::copy(&translated_src, &translated_dst).unwrap();

    // HACK
    // The following doesn't work because `include!()`ed contents can't start with inner attributes:
    //
    // ```
    // mod translated {
    //     include!(env!(TRANSLATED_DST));
    // }
    // ```
    //
    // Also, using the `sel4-mod-in-out-dir` crate requires] `#[feature(proc_macro_hygiene)]` (for
    // now).
    let translated_dst_str = format!("{}", translated_dst.display());
    let translated_contents_toks = quote! {
        #[path = #translated_dst_str]
        mod indirect;
        pub use indirect::*;
    };
    let translated_contents_dst = PathBuf::from(&out_dir).join("translated_contents.rs");
    fs::write(
        &translated_contents_dst,
        format!("{translated_contents_toks}"),
    )
    .unwrap();

    println!("cargo::rerun-if-env-changed={TRANSLATED_ENV}");
    println!("cargo::rerun-if-changed={translated_src}");
    println!(
        "cargo::rustc-env={TRANSLATED_CONTENTS_DST_ENV}={}",
        translated_contents_dst.display()
    );
}
