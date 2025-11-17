//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod cslot_allocator;
mod error;
mod hold_slots;
mod initialize;
mod lib_main;
mod memory;

#[cfg(not(feature = "alloc"))]
mod no_allocator;

#[sel4::sel4_cfg(all(ARCH_RISCV64, not(PT_LEVELS = "3")))]
compile_error!("unsupported configuration");
