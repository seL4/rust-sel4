//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_microkit::protection_domain;
use sel4_test_microkit::{embed_sdf_xml, match_handler};

embed_sdf_xml!("../../../x.system");

mod client;
mod server;

match_handler! {
    #[protection_domain(heap_size = 0x10_000)]
    fn init {
        "client" => client::init(),
        "server" => server::init(),
    }
}
