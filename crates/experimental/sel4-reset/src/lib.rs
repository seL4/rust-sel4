//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::arch::global_asm;
use core::slice;

use sel4_stack::{Stack, StackBottom};

mod rodata_var;

use rodata_var::rodata_var;

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
compile_error!("unsupported architecture");

// // //

#[repr(C)]
#[derive(Debug)]
struct RegionMeta {
    vaddr: usize,
    offset: usize,
    filesz: usize,
    memsz: usize,
}

struct Regions<'a> {
    meta: &'a [RegionMeta],
    data: &'a [u8],
}

impl Regions<'_> {
    unsafe fn reset_memory(&self) {
        for meta in self.meta {
            let dst = unsafe { slice::from_raw_parts_mut(meta.vaddr as *mut _, meta.memsz) };
            let (dst_data, dst_zero) = dst.split_at_mut(meta.filesz);
            let src_data = &self.data[meta.offset..][..meta.filesz];
            dst_data.copy_from_slice(src_data);
            dst_zero.fill(0);
        }
    }
}

unsafe fn get_regions() -> Regions<'static> {
    let start = *rodata_var!(sel4_reset_regions_start: usize);
    let meta_offset = *rodata_var!(sel4_reset_regions_meta_offset: usize);
    let meta_count = *rodata_var!(sel4_reset_regions_meta_count: usize);
    let data_offset = *rodata_var!(sel4_reset_regions_data_offset: usize);
    let data_size = *rodata_var!(sel4_reset_regions_data_size: usize);
    unsafe {
        Regions {
            meta: slice::from_raw_parts((start + meta_offset) as *const _, meta_count),
            data: slice::from_raw_parts((start + data_offset) as *const _, data_size),
        }
    }
}

// // //

unsafe extern "C" {
    fn _reset(x0: usize, x1: usize, x2: usize, x3: usize) -> !;
    fn _start(x0: usize, x1: usize, x2: usize, x3: usize) -> !;
}

pub fn reset() -> ! {
    unsafe {
        _reset(0, 0, 0, 0);
    }
}

pub fn reset1(x0: usize) -> ! {
    unsafe {
        _reset(x0, 0, 0, 0);
    }
}

pub fn reset2(x0: usize, x1: usize) -> ! {
    unsafe {
        _reset(x0, x1, 0, 0);
    }
}

pub fn reset3(x0: usize, x1: usize, x2: usize) -> ! {
    unsafe {
        _reset(x0, x1, x2, 0);
    }
}

pub fn reset4(x0: usize, x1: usize, x2: usize, x3: usize) -> ! {
    unsafe {
        _reset(x0, x1, x2, x3);
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn __sel4_reset__rust_entrypoint(
    x0: usize,
    x1: usize,
    x2: usize,
    x3: usize,
) -> ! {
    unsafe {
        get_regions().reset_memory();
        _start(x0, x1, x2, x3)
    }
}

const STACK_SIZE: usize = 4096;

#[unsafe(link_section = ".persistent")]
static STACK: Stack<STACK_SIZE> = Stack::new();

#[unsafe(no_mangle)]
static __sel4_reset__stack_bottom: StackBottom = STACK.bottom();

global_asm! {
    r#"
        .extern __sel4_reset__stack_bottom
        .extern __sel4_reset__rust_entrypoint

        .global _reset

        .section .text

        _reset:
    "#,
    #[cfg(target_arch = "aarch64")]
    r#"
            ldr x9, =__sel4_reset__stack_bottom
            ldr x9, [x9]
            mov sp, x9
            b __sel4_reset__rust_entrypoint
    "#,
    #[cfg(target_arch = "x86_64")]
    r#"
            mov rsp, __sel4_reset__stack_bottom
            mov rbp, rsp
            sub rsp, 0x8 // Stack must be 16-byte aligned before call
            push rbp
            jmp __sel4_reset__rust_entrypoint
    "#,
}
