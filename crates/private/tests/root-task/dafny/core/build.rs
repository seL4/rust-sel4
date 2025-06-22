//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env;
use std::fs;
use std::path::PathBuf;

const TRANSLATED_ENV: &str = "TRANSLATED";

fn main() {
    let translated_src = env::var(TRANSLATED_ENV).unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    let translated_dst = PathBuf::from(&out_dir).join("translated.rs");
    fs::copy(&translated_src, translated_dst).unwrap();

    println!("cargo::rerun-if-env-changed={TRANSLATED_ENV}");
    println!("cargo::rerun-if-changed={translated_src}");
}
