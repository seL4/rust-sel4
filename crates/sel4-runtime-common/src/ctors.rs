//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// See https://maskray.me/blog/2021-11-07-init-ctors-init-array

use core::mem;
use core::ptr;
use core::slice;

use sel4_panicking_env::abort;

type ArrayEntry = unsafe extern "C" fn();

extern "C" {
    static __preinit_array_start: ArrayEntry;
    static __preinit_array_end: ArrayEntry;
    static __init_array_start: ArrayEntry;
    static __init_array_end: ArrayEntry;
    static __fini_array_start: ArrayEntry;
    static __fini_array_end: ArrayEntry;

    fn _init();
    fn _fini();
}

mod _weak {
    #[linkage = "weak"]
    #[no_mangle]
    extern "C" fn _init() {}

    #[linkage = "weak"]
    #[no_mangle]
    extern "C" fn _fini() {}
}

unsafe fn run_array(start_addr: usize, end_addr: usize, section_name: &str) {
    if start_addr != end_addr {
        if start_addr % mem::size_of::<ArrayEntry>() != 0
            || end_addr % mem::size_of::<ArrayEntry>() != 0
        {
            abort!("{section_name:?} section is not properly aligned");
        }

        let len = (end_addr - start_addr) / mem::size_of::<ArrayEntry>();
        let array = slice::from_raw_parts(start_addr as *const ArrayEntry, len);
        for entry in array {
            (entry)();
        }
    }
}

fn run_preinit_array() {
    unsafe {
        run_array(
            ptr::addr_of!(__preinit_array_start) as usize,
            ptr::addr_of!(__preinit_array_end) as usize,
            ".preinit_array",
        )
    }
}

fn run_init_array() {
    unsafe {
        run_array(
            ptr::addr_of!(__init_array_start) as usize,
            ptr::addr_of!(__init_array_end) as usize,
            ".init_array",
        )
    }
}

fn run_fini_array() {
    unsafe {
        run_array(
            ptr::addr_of!(__fini_array_start) as usize,
            ptr::addr_of!(__fini_array_end) as usize,
            ".fini_array",
        )
    }
}

fn run_init() {
    unsafe { _init() }
}

fn run_fini() {
    unsafe { _fini() }
}

pub fn run_ctors() {
    run_preinit_array();
    run_init();
    run_init_array();
}

pub fn run_dtors() {
    run_fini_array();
    run_fini();
}
