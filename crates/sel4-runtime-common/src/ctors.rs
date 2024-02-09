//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::mem;
use core::ptr;
use core::slice;

use sel4_panicking_env::abort;

type Ctor = unsafe extern "C" fn();

extern "C" {
    static __init_array_start: Ctor;
    static __init_array_end: Ctor;
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn run_ctors() {
    let start = ptr::addr_of!(__init_array_start);
    let end = ptr::addr_of!(__init_array_end);

    // Cast to usize for comparison, otherwise rustc seems to apply an erroneous optimization
    // assuming __init_array_start != __init_array_end.
    if start as usize != end as usize {
        if start.align_offset(mem::size_of::<Ctor>()) != 0
            || end.align_offset(mem::size_of::<Ctor>()) != 0
        {
            abort!("'.init_array' section is not properly aligned");
        }

        let len = (end as usize - start as usize) / mem::size_of::<Ctor>();
        let ctors = slice::from_raw_parts(start, len);
        for ctor in ctors {
            (ctor)();
        }
    }
}
