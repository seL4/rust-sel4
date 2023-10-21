//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs1v15::{Signature, SigningKey};
use rsa::sha2::Sha256;
use rsa::signature::Signer;
use rsa::RsaPrivateKey;

const PRIV_KEY_PEM: &str = include_str!(concat!(env!("OUT_DIR"), "/priv.pem"));

fn get_priv_key() -> RsaPrivateKey {
    RsaPrivateKey::from_pkcs1_pem(PRIV_KEY_PEM).unwrap()
}

pub(crate) fn sign(data: &[u8]) -> Signature {
    let signing_key = SigningKey::<Sha256>::new_with_prefix(get_priv_key());
    signing_key.sign(data)
}
