//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(not(feature = "alloc"))]
use sel4_no_allocator as _;

mod cslot_allocator;
mod error;
mod hold_slots;
mod initialize;
mod lib_main;
mod memory;

#[sel4::sel4_cfg(all(ARCH_RISCV64, not(PT_LEVELS = "3")))]
compile_error!("unsupported configuration");
