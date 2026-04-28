//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::arch::global_asm;
use core::ptr;
use core::slice;

use sel4_stack::{Stack, StackBottom};

#[cfg(not(any(target_arch = "aarch64",)))]
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
            unsafe {
                ptr::write_bytes(dst_zero.as_mut_ptr(), 0, dst_zero.len());
            }
        }
    }
}

// // //

const STACK_SIZE: usize = 4096;

#[unsafe(link_section = ".persistent")]
static STACK: Stack<STACK_SIZE> = Stack::new();

#[unsafe(no_mangle)]
static __sel4_reset__stack_bottom: StackBottom = STACK.bottom();

// // //

#[unsafe(no_mangle)]
unsafe extern "C" fn __sel4_reset__reset_memory() {
    unsafe {
        get_regions().reset_memory();
    }
}

unsafe fn get_regions() -> Regions<'static> {
    unsafe {
        Regions {
            meta: slice::from_raw_parts(
                (sel4_reset_regions_start + sel4_reset_regions_meta_offset) as *const _,
                sel4_reset_regions_meta_count,
            ),
            data: slice::from_raw_parts(
                (sel4_reset_regions_start + sel4_reset_regions_data_offset) as *const _,
                sel4_reset_regions_data_size,
            ),
        }
    }
}

// HACK to force variables into .rodata without causing .rodata to end up in a PF_W segment
macro_rules! rodata {
    ($ident:ident) => {
        unsafe extern "C" {
            static $ident: usize;
        }
        global_asm! {
            r#"
                .section .rodata
            "#,
            #[cfg(target_pointer_width = "64")]
            r#"
                    .align 8
            "#,
            #[cfg(target_pointer_width = "32")]
            r#"
                    .align 4
            "#,
            r#"
                .global {ident}
                {ident}:
            "#,
            #[cfg(target_pointer_width = "64")]
            r#"
                    .quad 0
            "#,
            #[cfg(target_pointer_width = "32")]
            r#"
                    .word 0
            "#,
            r#"
                .size {ident}, .-{ident}
            "#,
            ident = sym $ident,
        }
    };
}

rodata!(sel4_reset_regions_start);
rodata!(sel4_reset_regions_meta_offset);
rodata!(sel4_reset_regions_meta_count);
rodata!(sel4_reset_regions_data_offset);
rodata!(sel4_reset_regions_data_size);

// // //

pub fn reset() -> ! {
    unsafe {
        _reset();
    }
    unreachable!()
}

unsafe extern "C" {
    fn _reset();
}

global_asm! {
    r#"
        .extern _start

        .global _reset

        .section .text

        _reset:
    "#,
    #[cfg(target_arch = "aarch64")]
    r#"
            ldr x9, =__sel4_reset__stack_bottom
            ldr x9, [x9]
            mov sp, x9
            bl __sel4_reset__reset_memory
            b _start

        1:  b 1b
    "#,
}
