//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

pub use sel4_test_sentinels::indicate_success;

#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".sel4_test_kind")]
pub static sel4_test_kind_root_task: () = ();
