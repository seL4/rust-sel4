#![no_std]

pub const CA_LIST: &[u8] = concat!(include_str!("cacert.pem"), "\0").as_bytes();
