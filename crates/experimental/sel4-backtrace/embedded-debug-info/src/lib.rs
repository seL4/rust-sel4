//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use sel4_backtrace_addr2line_context_helper::{Context, Error, new_context};
use sel4_phdrs::{PT_SEL4_EMBEDDED_DEBUG_INFO, locate_phdrs};

use sel4_phdrs_patched as _;

pub fn get_context() -> Result<Context, Error> {
    let embedded_debug_info = unsafe {
        locate_phdrs()
            .unwrap()
            .find_by_type(PT_SEL4_EMBEDDED_DEBUG_INFO)
            .unwrap()
            .bytes()
    };
    let obj = object::File::parse(embedded_debug_info).unwrap();
    new_context(&obj)
}
