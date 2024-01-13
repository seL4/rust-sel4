//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::arch::{asm, global_asm};
use core::ffi::c_void;
use core::slice;

// TODO
// Use overflow checking arithmetic ops, and abort!() on overflow

// This is enforced by AArch64 and x86_64
const STACK_ALIGNMENT: usize = 16;

// 16 bytes for ELF-level "thread control block"
// https://akkadia.org/drepper/tls.pdf
const RESERVED_ABOVE_THREAD_POINTER: usize = 16;

pub type SetThreadPointerFn = unsafe extern "C" fn(thread_pointer: usize);
pub type ContFn = unsafe extern "C" fn(cont_arg: ContArg) -> !;
pub type ContArg = *mut c_void;

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsImage {
    pub vaddr: usize,
    pub filesz: usize,
    pub memsz: usize,
    pub align: usize,
}

impl TlsImage {
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn initialize_on_stack_and_continue(
        &self,
        set_thread_pointer_fn: SetThreadPointerFn,
        cont_fn: ContFn,
        cont_arg: ContArg,
    ) -> ! {
        let args = InternalContArgs {
            tls_image: self as *const TlsImage,
            set_thread_pointer_fn,
            cont_fn,
            cont_arg,
        };
        let segment_size = self.memsz;
        let segment_align_down_mask = !(self.align - 1);
        let reserved_above_thread_pointer = RESERVED_ABOVE_THREAD_POINTER;
        let stack_align_down_mask = !(STACK_ALIGNMENT - 1);
        __sel4_runtime_initialize_tls_and_continue(
            &args as *const InternalContArgs,
            segment_size,
            segment_align_down_mask,
            reserved_above_thread_pointer,
            stack_align_down_mask,
            continue_with,
        )
    }

    fn tls_base_addr(&self, thread_pointer: usize) -> usize {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "aarch64")] {
                (thread_pointer + RESERVED_ABOVE_THREAD_POINTER).next_multiple_of(self.align)
            } else if #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))] {
                thread_pointer.next_multiple_of(self.align)
            } else if #[cfg(target_arch = "x86_64")] {
                (thread_pointer - self.memsz) & !(self.align - 1)
            } else {
                compile_error!("unsupported architecture");
            }
        }
    }

    unsafe fn init(&self, thread_pointer: usize) {
        let addr = self.tls_base_addr(thread_pointer);
        let window = slice::from_raw_parts_mut(addr as *mut _, self.memsz);
        let (tdata, tbss) = window.split_at_mut(self.filesz);
        tdata.copy_from_slice(self.data());
        tbss.fill(0);
    }

    unsafe fn data(&self) -> &'static [u8] {
        slice::from_raw_parts(self.vaddr as *mut _, self.filesz)
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct InternalContArgs {
    tls_image: *const TlsImage,
    set_thread_pointer_fn: SetThreadPointerFn,
    cont_fn: ContFn,
    cont_arg: ContArg,
}

extern "C" {
    fn __sel4_runtime_initialize_tls_and_continue(
        args: *const InternalContArgs,
        segment_size: usize,
        segment_align_down_mask: usize,
        reserved_above_thread_pointer: usize,
        stack_align_down_mask: usize,
        continue_with: unsafe extern "C" fn(
            args: *const InternalContArgs,
            thread_pointer: usize,
        ) -> !,
    ) -> !;
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "aarch64")] {
        global_asm! {
            r#"
                .global __sel4_runtime_initialize_tls_and_continue

                .section .text

                __sel4_runtime_initialize_tls_and_continue:
                    mov x9, sp
                    sub x9, x9, x1 // x1: segment_size
                    and x9, x9, x2 // x2: segment_align_down_mask
                    sub x9, x9, x3 // x3: reserved_above_thread_pointer
                    and x9, x9, x2 // x2: segment_align_down_mask
                    mov x10, x9    // save thread pointer for later
                    and x9, x9, x4 // x4: stack_align_down_mask
                    mov sp, x9
                    mov x1, x10    // pass thread pointer to continuation
                    br x5
            "#
        }
    } else if #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))] {
        global_asm! {
            r#"
                .global __sel4_runtime_initialize_tls_and_continue

                .section .text

                __sel4_runtime_initialize_tls_and_continue:
                    mv s9, sp
                    sub s9, s9, a1 // a1: segment_size
                    and s9, s9, a2 // a2: segment_align_down_mask
                    sub s9, s9, a3 // a3: reserved_above_thread_pointer
                    and s9, s9, a2 // a2: segment_align_down_mask
                    mv s10, s9     // save thread pointer for later
                    and s9, s9, a4 // a4: stack_align_down_mask
                    mv sp, s9
                    mv a1, s10     // pass thread pointer to continuation
                    jr a5
            "#
        }
    } else if #[cfg(target_arch = "x86_64")] {
        global_asm! {
            r#"
                .global __sel4_runtime_initialize_tls_and_continue

                .section .text

                __sel4_runtime_initialize_tls_and_continue:
                    mov r10, rsp
                    sub r10, 0x8 // space for thread structure
                    and r10, rdx // rdx: segment_align_down_mask
                    mov r11, r10 // save thread_pointer for later
                    sub r10, rsi // rsi: segment_size
                    and r10, rdx // rdx: segment_align_down_mask
                    and r10, r8  // r8: stack_align_down_mask
                    mov rsp, r10
                    mov rsi, r11 // pass thread pointer to continuation
                    mov rbp, rsp
                    sub rsp, 0x8 // stack must be 16-byte aligned before call
                    push rbp
                    call r9
            "#
        }
    } else {
        compile_error!("unsupported architecture");
    }
}

unsafe extern "C" fn continue_with(args: *const InternalContArgs, thread_pointer: usize) -> ! {
    let args = unsafe { &*args };
    let tls_image = unsafe { &*args.tls_image };
    tls_image.init(thread_pointer);

    (args.set_thread_pointer_fn)(thread_pointer);

    if cfg!(target_arch = "x86_64") {
        (thread_pointer as *mut usize).write(thread_pointer);
    }

    (args.cont_fn)(args.cont_arg)
}

pub const DEFAULT_SET_THREAD_POINTER_FN: SetThreadPointerFn = default_set_thread_pointer;

#[cfg(target_arch = "aarch64")]
unsafe extern "C" fn default_set_thread_pointer(val: usize) {
    asm!("msr tpidr_el0, {val}", val = in(reg) val);
}

#[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))]
unsafe extern "C" fn default_set_thread_pointer(val: usize) {
    asm!("mv tp, {val}", val = in(reg) val);
}

#[cfg(target_arch = "x86_64")]
unsafe extern "C" fn default_set_thread_pointer(val: usize) {
    asm!("wrfsbase {val}", val = in(reg) val);
}
