#![no_std]
#![feature(int_roundings)]
#![feature(strict_provenance)]

use core::arch::{asm, global_asm};
use core::ffi::c_void;
use core::ptr;
use core::slice;

// NOTE
// This is enforced by AArch64 and x86_64
const STACK_ALIGNMENT: usize = 16;

// NOTE
// 16 bytes for ELF-level "thread control block"
// https://akkadia.org/drepper/tls.pdf
const RESERVED_ABOVE_TPIDR: usize = 16;

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
    pub unsafe fn reserve_on_stack_and_continue(&self, cont_fn: ContFn, cont_arg: ContArg) -> ! {
        let args = InternalContArgs {
            tls_image: self as *const TlsImage,
            cont_fn,
            cont_arg,
        };
        let segment_size = self.memsz;
        let segment_align_down_mask = !(self.align - 1);
        let reserved_above_tpidr = RESERVED_ABOVE_TPIDR;
        let stack_align_down_mask = !(STACK_ALIGNMENT - 1);
        __sel4_runtime_reserve_tls_and_continue(
            &args as *const InternalContArgs,
            segment_size,
            segment_align_down_mask,
            reserved_above_tpidr,
            stack_align_down_mask,
            continue_with,
        )
    }

    fn base_addr(&self, tpidr: usize) -> usize {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "aarch64")] {
                (tpidr + RESERVED_ABOVE_TPIDR).next_multiple_of(self.align)
            } else if #[cfg(target_arch = "x86_64")] {
                (tpidr - self.memsz) & !(self.align - 1)
            } else {
                compile_error!("unsupported architecture")
            }
        }
    }

    unsafe fn init(&self, tpidr: usize) {
        let addr = self.base_addr(tpidr);
        let window = slice::from_raw_parts_mut(ptr::from_exposed_addr_mut(addr), self.memsz);
        let (tdata, tbss) = window.split_at_mut(self.filesz);
        tdata.copy_from_slice(self.data());
        tbss.fill(0);
    }

    unsafe fn data(&self) -> &'static [u8] {
        slice::from_raw_parts(ptr::from_exposed_addr_mut(self.vaddr), self.filesz)
    }
}

pub type ContFn = unsafe extern "C" fn(*mut c_void) -> !;
pub type ContArg = *mut c_void;

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct InternalContArgs {
    tls_image: *const TlsImage,
    cont_fn: ContFn,
    cont_arg: ContArg,
}

extern "C" {
    fn __sel4_runtime_reserve_tls_and_continue(
        args: *const InternalContArgs,
        segment_size: usize,
        segment_align_down_mask: usize,
        reserved_above_tpidr: usize,
        stack_align_down_mask: usize,
        continue_with: unsafe extern "C" fn(args: *const InternalContArgs, tpidr: usize) -> !,
    ) -> !;
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "aarch64")] {
        global_asm! {
            r#"
                .global __sel4_runtime_reserve_tls_and_continue

                .section .text

                __sel4_runtime_reserve_tls_and_continue:
                    mov x9, sp
                    sub x9, x9, x1 // x1: segment_size
                    and x9, x9, x2 // x2: segment_align_down_mask
                    sub x9, x9, x3 // x3: reserved_above_tpidr
                    and x9, x9, x2 // x2: segment_align_down_mask
                    mov x10, x9    // save tpidr for later
                    and x9, x9, x4 // x4: stack_align_down_mask
                    mov sp, x9
                    mov x1, x10    // pass tpidr to continuation
                    br x5
            "#
        }
    } else if #[cfg(target_arch = "x86_64")] {
        global_asm! {
            r#"
            .global __sel4_runtime_reserve_tls_and_continue

            .section .text

            __sel4_runtime_reserve_tls_and_continue:
                mov r10, rsp
                sub r10, 0x8 // space for thread structure
                and r10, rdx // rdx: segment_align_down_mask
                mov r11, r10 // save tpidr for later
                sub r10, rsi // rsi: segment_size
                and r10, rdx // rdx: segment_align_down_mask
                and r10, r8  // r8: stack_align_down_mask
                mov rsp, r10
                mov rsi, r11 // pass tpidr to continuation
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

unsafe extern "C" fn continue_with(args: *const InternalContArgs, tpidr: usize) -> ! {
    let args = unsafe { &*args };
    let tls_image = unsafe { &*args.tls_image };
    tls_image.init(tpidr);

    set_tls_base(tpidr);

    if cfg!(target_arch = "x86_64") {
        ptr::from_exposed_addr_mut::<usize>(tpidr).write(tpidr);
    }

    (args.cont_fn)(args.cont_arg)
}

// helpers

// NOTE
// The Rust optimizer has caused issues here. For example, thread local accesses
// below being emitted before this call. Fences in core::sync::atomic didn't work,
// but a get_tls_base() call did. For now, making this function #[inline(never)] seems
// to be sufficient.

cfg_if::cfg_if! {
    if #[cfg(target_arch = "aarch64")] {

        #[inline(never)] // issues with optimizer
        unsafe fn set_tls_base(tpidr: usize) {
            asm!("msr tpidr_el0, {tpidr}", tpidr = in(reg) tpidr);
        }

        #[allow(dead_code)]
        #[inline(never)] // issues with optimizer
        unsafe fn get_tls_base() -> usize {
            let mut tpidr;
            asm!("mrs {tpidr}, tpidr_el0", tpidr = out(reg) tpidr);
            tpidr
        }

    } else if #[cfg(target_arch = "x86_64")] {
        sel4::sel4_cfg_if! {
            if #[cfg(FSGSBASE_INST)] {

                #[inline(never)] // issues with optimizer
                unsafe fn set_tls_base(val: usize) {
                    asm!("wrfsbase {val}", val = in(reg) val);
                }

                #[allow(dead_code)]
                #[inline(never)] // issues with optimizer
                unsafe fn get_tls_base() -> usize {
                    let mut val;
                    asm!("rdfsbase {val}", val = out(reg) val);
                    val
                }

            } else if #[cfg(SET_TLS_BASE_SELF)] {

                unsafe fn set_tls_base(val: usize) {
                    sel4::sys::seL4_SetTLSBase(val.try_into().unwrap());
                }

                #[allow(dead_code)]
                #[inline(never)] // issues with optimizer
                unsafe fn get_tls_base() -> usize {
                    let mut val;
                    asm!("mov {val}, fs:0", val = out(reg) val);
                    val
                }

            } else {
                compile_error!("unsupported configuraton");
            }
        }
    } else {

        compile_error!("unsupported architecture");

    }
}
