//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

pub fn run_test_main() {
    unsafe {
        main(0, core::ptr::null());
    }
}

unsafe extern "C" {
    fn main(argc: isize, argv: *const *const u8);
}

// HACK
trait IsUnit {}

impl IsUnit for () {}

#[lang = "start"]
fn lang_start<T: IsUnit>(
    main: fn() -> T,
    _argc: isize,
    _argv: *const *const u8,
    _sigpipe: u8,
) -> isize {
    main();
    0
}
