//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::arch::{asm, global_asm};
use core::ptr;
use core::slice;

use cfg_if::cfg_if;

// NOTE
//
// - aarch64 and arm use variant 1 defined in [1][2].
// - x86_64 uses variant 2 defined in [1][2].
// - riscv uses variant 1 with a twist: the thread pointer points to the first address _past_ the
//   TCB [3].
//
// [1] https://akkadia.org/drepper/tls.pdf
// [2] https://fuchsia.dev/fuchsia-src/development/kernel/threads/tls#implementation
// [3] https://github.com/riscv-non-isa/riscv-elf-psabi-doc/blob/master/riscv-elf.adoc#thread-local-storage

const STACK_ALIGNMENT: usize = {
    cfg_if! {
        if #[cfg(any(
            target_arch = "aarch64",
            target_arch = "riscv32",
            target_arch = "riscv64",
            target_arch = "x86_64",
        ))] {
            16
        } else if #[cfg(target_arch = "arm")] {
            8
        } else {
            compile_error!("unsupported architecture")
        }
    }
};

pub type SetThreadPointerFn = unsafe extern "C" fn(thread_pointer: usize);

pub type ContFn = unsafe extern "C" fn(cont_arg: *mut ContArg) -> !;

pub enum ContArg {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UncheckedTlsImage {
    pub vaddr: usize,
    pub filesz: usize,
    pub memsz: usize,
    pub align: usize,
}

impl UncheckedTlsImage {
    pub fn check(&self) -> Option<TlsImage> {
        if self.memsz >= self.filesz && self.align.is_power_of_two() && self.align > 0 {
            Some(TlsImage {
                vaddr: self.vaddr,
                filesz: self.filesz,
                memsz: self.memsz,
                align: self.align,
            })
        } else {
            None
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsImage {
    vaddr: usize,
    filesz: usize,
    memsz: usize,
    align: usize,
}

impl TlsImage {
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn initialize_on_stack_and_continue(
        &self,
        set_thread_pointer_fn: SetThreadPointerFn,
        cont_fn: ContFn,
        cont_arg: *mut ContArg,
    ) -> ! {
        let args = InternalContArgs {
            tls_image: ptr::addr_of!(*self),
            set_thread_pointer_fn,
            cont_fn,
            cont_arg,
        };
        let segment_size = self.memsz;
        let segment_align_down_mask = !(self.align - 1);
        let stack_align_down_mask = !(STACK_ALIGNMENT - 1);
        __sel4_initialize_tls_on_stack__reserve(
            ptr::addr_of!(args),
            segment_size,
            segment_align_down_mask,
            stack_align_down_mask,
        )
    }

    unsafe fn continue_with(
        &self,
        set_thread_pointer_fn: SetThreadPointerFn,
        cont_fn: ContFn,
        cont_arg: *mut ContArg,
        thread_pointer: usize,
        tls_base_addr: usize,
    ) -> ! {
        self.initialize(tls_base_addr);

        if cfg!(target_arch = "x86_64") {
            (thread_pointer as *mut usize).write(thread_pointer);
        }

        (set_thread_pointer_fn)(thread_pointer);

        (cont_fn)(cont_arg)
    }

    unsafe fn initialize(&self, tls_base_addr: usize) {
        let image_data_window = slice::from_raw_parts(self.vaddr as *mut u8, self.filesz);
        let tls_window = slice::from_raw_parts_mut(tls_base_addr as *mut u8, self.memsz);
        let (tdata, tbss) = tls_window.split_at_mut(self.filesz);
        tdata.copy_from_slice(image_data_window);
        tbss.fill(0);
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct InternalContArgs {
    tls_image: *const TlsImage,
    set_thread_pointer_fn: SetThreadPointerFn,
    cont_fn: ContFn,
    cont_arg: *mut ContArg,
}

#[no_mangle]
unsafe extern "C" fn __sel4_initialize_tls_on_stack__continue(
    args: *const InternalContArgs,
    thread_pointer: usize,
    tls_base_addr: usize,
) -> ! {
    let args = args.as_ref().unwrap();
    let tls_image = args.tls_image.as_ref().unwrap();
    tls_image.continue_with(
        args.set_thread_pointer_fn,
        args.cont_fn,
        args.cont_arg,
        thread_pointer,
        tls_base_addr,
    )
}

extern "C" {
    fn __sel4_initialize_tls_on_stack__reserve(
        args: *const InternalContArgs,
        segment_size: usize,
        segment_align_down_mask: usize,
        stack_align_down_mask: usize,
    ) -> !;
}

macro_rules! common_asm {
    () => {
        r#"
            .extern __sel4_initialize_tls_on_stack__continue

            .global __sel4_initialize_tls_on_stack__reserve

            .section .text

            __sel4_initialize_tls_on_stack__reserve:
        "#
    };
}

cfg_if! {
    if #[cfg(target_arch = "aarch64")] {
        global_asm! {
            common_asm!(),
            r#"
                    mov x9, sp
                    sub x9, x9, x1  // x1: segment_size
                    and x9, x9, x2  // x2: segment_align_down_mask
                    mov x10, x9     // save tls_base_addr for later
                    sub x9, x9, #16 // reserve for TCB
                    and x9, x9, x2  // x2: segment_align_down_mask
                    mov x11, x9     // save thread_pointer for later
                    and x9, x9, x3  // x3: stack_align_down_mask
                    mov sp, x9
                    mov x1, x11     // pass thread_pointer to continuation
                    mov x2, x10     // pass tls_base_addr to continuation
                    b __sel4_initialize_tls_on_stack__continue
            "#
        }
    } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
        global_asm! {
            common_asm!(),
            r#"
                    mv t0, sp
                    sub t0, t0, a1  // a1: segment_size
                    and t0, t0, a2  // a2: segment_align_down_mask
                    mv t1, t0       // save thread_pointer and tls_base_addr, which are equal, for later
                    and t0, t0, a3  // a3: stack_align_down_mask
                    mv sp, t0
                    mv a1, t1       // pass thread_pointer to continuation
                    mv a2, t1       // pass tls_base_addr to continuation
                    j __sel4_initialize_tls_on_stack__continue
            "#
        }
    } else if #[cfg(target_arch = "x86_64")] {
        global_asm! {
            common_asm!(),
            r#"
                    mov r10, rsp
                    sub r10, 0x8    // reserve for TCB (TODO is 8 bytes enough?)
                    and r10, rdx    // rdx: segment_align_down_mask
                    mov r11, r10    // save thread_pointer for later
                    sub r10, rsi    // rsi: segment_size
                    and r10, rdx    // rdx: segment_align_down_mask
                    mov rax, r10    // save tls_base_addr for later
                    and r10, rcx    // rcx: stack_align_down_mask
                    mov rsp, r10
                    mov rsi, r11    // pass thread_pointer to continuation
                    mov rdx, rax    // pass tls_base_addr to continuation
                    mov rbp, rsp
                    sub rsp, 0x8    // stack must be 16-byte aligned before call
                    push rbp
                    call __sel4_initialize_tls_on_stack__continue
            "#
        }
    } else {
        compile_error!("unsupported architecture");
    }
}

pub const DEFAULT_SET_THREAD_POINTER_FN: SetThreadPointerFn = default_set_thread_pointer;

unsafe extern "C" fn default_set_thread_pointer(thread_pointer: usize) {
    let val = thread_pointer;

    cfg_if! {
        if #[cfg(target_arch = "aarch64")] {
            asm!("msr tpidr_el0, {val}", val = in(reg) val);
        } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
            asm!("mv tp, {val}", val = in(reg) val);
        } else if #[cfg(target_arch = "x86_64")] {
            asm!("wrfsbase {val}", val = in(reg) val);
        } else {
            compile_error!("unsupported architecture");
        }
    }
}
