//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

pub const CA_LIST: &[u8] = concat!(include_str!("cacert.pem"), "\0").as_bytes();
