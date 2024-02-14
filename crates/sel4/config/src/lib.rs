//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

pub use sel4_config_macros::*;

pub mod consts {
    //! The kernel configuration as `const` items.
    //!
    //! While this module can be used as an alternative to the `sel4_cfg_*!` macros for accessing
    //! the kernel configuration at the value level, its primary purpose is to provide a reference
    //! within Rustdoc for the active configuration. Towards that end, the generated source of this
    //! module is also provided in this module's Rustdoc to make browsing easier.
    #![doc = concat!("```rust\n", include_str!(concat!(env!("OUT_DIR"), "/consts_gen.rs")), "```\n")]

    include!(concat!(env!("OUT_DIR"), "/consts_gen.rs"));
}
