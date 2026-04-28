//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_microkit::{Never, protection_domain};

sel4_test_microkit::embed_sdf_script!("system.xml");

#[protection_domain]
fn init() -> Never {
    sel4_test_microkit::indicate_success()
}
