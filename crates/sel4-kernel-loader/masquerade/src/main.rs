//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// RUSTFLAGS="-Crelocation-model=pie"
// -Z build-std=core,compiler_builtins (to ensure all is pie)
// --release

#![no_std]
#![no_main]

use core::fmt::Write;
use core::mem;
use core::ptr;

use fdt::Fdt;

// TODO
// extern crate sel4_no_panic;

mod caches;
mod dbg;
mod payload;
mod rt;

use dbg::D;
use payload::Payload;

extern "C" fn main(dtb_addr: usize) {
    inspect_dtb(dtb_addr);
    let payload = get_payload();
    let entry_addr = unsafe { payload.deploy() };
    let entry_fn = unsafe { mem::transmute::<usize, EntryFn>(entry_addr) };
    (entry_fn)(dtb_addr)
}

type EntryFn = extern "C" fn(usize) -> !;

fn get_payload() -> Payload {
    unsafe { Payload::deserialize(ptr::addr_of!(_payload_start)) }
}

unsafe extern "C" {
    static _payload_start: u8;
}

fn inspect_dtb(dtb_addr: usize) {
    let fdt = unsafe { Fdt::from_ptr(dtb_addr as *const u8) }.unwrap_or_else(|err| panic!("{err}"));
    for r in fdt.memory().regions() {
        writeln!(D, "region: {:#x}", r.starting_address.addr()).unwrap();
    }
}
