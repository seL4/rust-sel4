//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env;
use std::fs;
use std::path::PathBuf;

use rsa::pkcs1::EncodeRsaPrivateKey;

const RSA_KEY_SIZE: usize = 2048;

fn main() {
    let priv_key = rsa::RsaPrivateKey::new(&mut rsa::rand_core::OsRng, RSA_KEY_SIZE).unwrap();
    let priv_key_pem = priv_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF).unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir).join("priv.pem");
    fs::write(out_path, &priv_key_pem).unwrap();

    // No external dependencies
    println!("cargo:rerun-if-changed=build.rs");
}
