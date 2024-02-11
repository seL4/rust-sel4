//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

pub use sel4_config_macros::*;

pub mod consts {
    #![doc = concat!("```rust\n", include_str!(concat!(env!("OUT_DIR"), "/consts_gen.rs")), "```\n")]

    include!(concat!(env!("OUT_DIR"), "/consts_gen.rs"));
}
