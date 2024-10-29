//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env;
use std::fs;
use std::path::PathBuf;

use rcgen::generate_simple_self_signed;

fn main() {
    let subject_alt_names = vec!["localhost".to_string()];
    let cert = generate_simple_self_signed(subject_alt_names).unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    let cert_path = PathBuf::from(&out_dir).join("cert.pem");
    fs::write(cert_path, cert.cert.pem()).unwrap();
    let priv_path = PathBuf::from(&out_dir).join("priv.pem");
    fs::write(priv_path, cert.key_pair.serialize_pem()).unwrap();
}
