//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::arch::naked_asm;
use core::slice;

use sel4_panicking_env::abort;
use sel4_phdrs::{PT_SEL4_RESET_REGIONS, locate_phdrs};
use sel4_stack::{Stack, StackBottom};

use sel4_phdrs_patched as _;

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
    unsafe {
        let phdr = locate_phdrs()
            .unwrap_or_else(|err| abort!("{err}"))
            .find_by_type(PT_SEL4_RESET_REGIONS)
            .unwrap_or_else(|| abort!("missing PT_SEL4_RESET_REGIONS program header"));
        slice::from_raw_parts(
            phdr.p_vaddr as *const _,
            phdr.p_memsz / size_of::<RegionMeta>(),
        )
    }
}

// // //

unsafe extern "C" {
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

unsafe extern "C" fn reset_rust_entrypoint(x0: usize, x1: usize, x2: usize, x3: usize) -> ! {
    unsafe {
        reset_memory(get_regions());
        _start(x0, x1, x2, x3)
    }
}

const STACK_SIZE: usize = 4096;

#[unsafe(link_section = ".persistent")]
static STACK: Stack<STACK_SIZE> = Stack::new();

static STACK_BOTTOM: StackBottom = STACK.bottom();

#[unsafe(naked)]
unsafe extern "C" fn _reset(x0: usize, x1: usize, x2: usize, x3: usize) -> ! {
    naked_asm! {
        cfg_select! {
            target_arch = "aarch64" => r#"
                ldr x9, ={reset_stack_bottom}
                ldr x9, [x9]
                mov sp, x9
                b {reset_rust_entrypoint}
            "#,
            target_arch = "arm" => r#"
                ldr r8, ={reset_stack_bottom}
                ldr r8, [r8]
                mov sp, r8
                b {reset_rust_entrypoint}
            "#,
            target_arch = "riscv64" => r#"
                la sp, {reset_stack_bottom}
                ld sp, (sp)
                j {reset_rust_entrypoint}
            "#,
            target_arch = "riscv32" => r#"
                la sp, {reset_stack_bottom}
                lw sp, (sp)
                j {reset_rust_entrypoint}
            "#,
            target_arch = "x86_64" => r#"
                mov rsp, {reset_stack_bottom}
                mov rbp, rsp
                sub rsp, 0x8 // Stack must be 16-byte aligned before call
                push rbp
                jmp {reset_rust_entrypoint}
            "#,
        },
        reset_stack_bottom = sym STACK_BOTTOM,
        reset_rust_entrypoint = sym reset_rust_entrypoint,
    }
}
