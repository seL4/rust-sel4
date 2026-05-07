//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

const PT_LOOS: u32 = 0x6000_0000;

const PT_SEL4_BASE: u32 = PT_LOOS + 0x4c3_4000; // "L4" -> 0x4c34

pub const PT_SEL4_RESET_REGIONS: u32 = PT_SEL4_BASE + 1;
pub const PT_SEL4_EMBEDDED_DEBUG_INFO: u32 = PT_SEL4_BASE + 2;
pub const PT_SEL4_CAPDL_SPEC: u32 = PT_SEL4_BASE + 3;
pub const PT_SEL4_CAPDL_FRAME_DATA: u32 = PT_SEL4_BASE + 4;
pub const PT_SEL4_KERNEL_LOADER_PAYLOAD: u32 = PT_SEL4_BASE + 5;
