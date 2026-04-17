//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::arch::global_asm;
use core::slice;

use sel4_rodata_static::rodata_static;
use sel4_stack::{Stack, StackBottom};

#[cfg(not(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "riscv64",
    target_arch = "riscv32",
    target_arch = "x86_64",
)))]
compile_error!("unsupported architecture");

// // //

#[repr(C)]
#[derive(Debug)]
struct RegionMeta {
    dst_vaddr: usize,
    dst_size: usize,
    src_vaddr: usize,
    src_size: usize,
}

unsafe fn reset_memory(regions: &'static [RegionMeta]) {
    for meta in regions {
        let dst = unsafe { slice::from_raw_parts_mut(meta.dst_vaddr as *mut u8, meta.dst_size) };
        let (dst_data, dst_zero) = dst.split_at_mut(meta.src_size);
        if meta.src_vaddr != 0 {
            let src =
                unsafe { slice::from_raw_parts_mut(meta.src_vaddr as *mut u8, meta.src_size) };
            dst_data.copy_from_slice(src);
        }
        dst_zero.fill(0);
    }
}

unsafe fn get_regions() -> &'static [RegionMeta] {
    let meta_vaddr = *rodata_static!(sel4_reset_regions_meta_vaddr: usize);
    let meta_count = *rodata_static!(sel4_reset_regions_meta_count: usize);
    unsafe { slice::from_raw_parts(meta_vaddr as *const _, meta_count) }
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
        reset_memory(get_regions());
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

        .section .text.reset, "axR", %progbits
        _reset:
    "#,
    #[cfg(target_arch = "aarch64")]
    r#"
            ldr x9, =__sel4_reset__stack_bottom
            ldr x9, [x9]
            mov sp, x9
            b __sel4_reset__rust_entrypoint
    "#,
    #[cfg(target_arch = "arm")]
    r#"
            ldr r8, =__sel4_reset__stack_bottom
            ldr r8, [r8]
            mov sp, r8
            b __sel4_reset__rust_entrypoint
    "#,
    #[cfg(target_arch = "riscv64")]
    r#"
            la sp, __sel4_reset__stack_bottom
            ld sp, (sp)
            j __sel4_reset__rust_entrypoint
    "#,
    #[cfg(target_arch = "riscv32")]
    r#"
            la sp, __sel4_reset__stack_bottom
            lw sp, (sp)
            j __sel4_reset__rust_entrypoint
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
