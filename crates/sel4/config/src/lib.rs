//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

pub use sel4_config_macros::*;

pub mod consts {
    include!(concat!(env!("OUT_DIR"), "/consts_gen.rs"));
}
