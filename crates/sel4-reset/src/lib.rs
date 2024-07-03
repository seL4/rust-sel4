//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::arch::global_asm;
use core::cell::UnsafeCell;
use core::ptr;
use core::slice;

use cfg_if::cfg_if;

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

impl<'a> Regions<'a> {
    unsafe fn reset_memory(&self) {
        for meta in self.meta {
            let dst = unsafe { slice::from_raw_parts_mut(meta.vaddr as *mut _, meta.memsz) };
            let (dst_data, dst_zero) = dst.split_at_mut(meta.filesz);
            let src_data = &self.data[meta.offset..][..meta.filesz];
            dst_data.copy_from_slice(src_data);
            ptr::write_bytes(dst_zero.as_mut_ptr(), 0, dst_zero.len());
        }
    }
}

// // //

#[repr(C)]
#[cfg_attr(
    any(
        target_arch = "aarch64",
        target_arch = "riscv32",
        target_arch = "riscv64",
        target_arch = "x86_64",
    ),
    repr(align(16))
)]
#[cfg_attr(target_arch = "arm", repr(align(4)))]
pub struct Stack<const N: usize>(UnsafeCell<[u8; N]>);

unsafe impl<const N: usize> Sync for Stack<N> {}

impl<const N: usize> Stack<N> {
    pub const fn new() -> Self {
        Self(UnsafeCell::new([0; N]))
    }

    pub const fn top(&self) -> StackTop {
        StackTop(self.0.get().cast::<u8>().wrapping_add(N))
    }
}

impl<const N: usize> Default for Stack<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(transparent)]
pub struct StackTop(#[allow(dead_code)] *mut u8);

unsafe impl Sync for StackTop {}

const STACK_SIZE: usize = 4096;

#[link_section = ".persistent"]
static STACK: Stack<STACK_SIZE> = Stack::new();

#[no_mangle]
static __sel4_reset__stack_top: StackTop = STACK.top();

// // //

#[no_mangle]
unsafe extern "C" fn __sel4_reset__reset_memory() {
    unsafe {
        get_regions().reset_memory();
    }
}

unsafe fn get_regions() -> Regions<'static> {
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

// HACK to force variables into .rodata without causing .rodata to end up in a PF_W segment
macro_rules! rodata {
    ($ident:ident) => {
        extern "C" {
            static $ident: usize;
        }
        global_asm! {
            ".section .rodata",
            concat!(".global ", stringify!($ident)),
            concat!(stringify!($ident), ": ", asm_word_size!(), " 0"),
            concat!(".size ", stringify!($ident), ", .-", stringify!($ident)),
        }
    };
}

cfg_if! {
    if #[cfg(target_pointer_width = "64")] {
        macro_rules! asm_word_size {
            () => {
                ".quad"
            }
        }
    } else if #[cfg(target_pointer_width = "32")] {
        macro_rules! asm_word_size {
            () => {
                ".word"
            }
        }
    } else {
        compile_error!("unsupported configuration");
    }
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

extern "C" {
    fn _reset();
}

macro_rules! common_asm_prefix {
    () => {
        r#"
            .extern _start

            .global _reset

            .section .text

            _reset:
        "#
    };
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "aarch64")] {
        global_asm! {
            common_asm_prefix!(),
            r#"
                    ldr x9, =__sel4_reset__stack_top
                    ldr x9, [x9]
                    mov sp, x9
                    bl __sel4_reset__reset_memory
                    b _start
        
                1:  b 1b
            "#
        }
    } else {
        compile_error!("unsupported architecture");
    }
}
