//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

mod buffers;
mod cslot_allocator;
mod error;
mod hold_slots;
mod initialize;
mod memory;

pub use buffers::{InitializerBuffers, PerObjectBuffer};
pub use error::CapDLInitializerError;
pub use initialize::Initializer;

#[sel4::sel4_cfg(all(ARCH_RISCV64, not(PT_LEVELS = "3")))]
compile_error!("unsupported configuration");
