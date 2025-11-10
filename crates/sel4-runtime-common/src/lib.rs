//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(cfg_target_thread_local)]
#![feature(never_type)]
#![feature(core_intrinsics)]
#![feature(linkage)]
#![allow(internal_features)]

use core::sync::atomic::{AtomicBool, Ordering};

use sel4_elf_header::{ElfHeader, ProgramHeader};
use sel4_panicking_env::abort;

mod abort;
mod start;

#[cfg(target_thread_local)]
mod tls;

#[cfg(panic = "unwind")]
mod unwinding;

#[cfg(not(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "riscv64",
    target_arch = "riscv32",
    target_arch = "x86_64",
)))]
compile_error!("unsupported architecture");

#[allow(clippy::missing_safety_doc)]
pub unsafe fn with_local_init(f: impl FnOnce() -> !) -> ! {
    cfg_if::cfg_if! {
        if #[cfg(target_thread_local)] {
            unsafe {
                tls::with_tls(f)
            }
        } else {
            f()
        }
    }
}

static GLOBAL_INIT_COMPLETE: AtomicBool = AtomicBool::new(false);

unsafe fn global_init() {
    #[cfg(panic = "unwind")]
    {
        unwinding::init_unwinding();
    }

    sel4_ctors_dtors::run_ctors().unwrap_or_else(|err| abort!("{err}"));

    GLOBAL_INIT_COMPLETE.swap(true, Ordering::Release);
}

pub fn global_init_complete() -> bool {
    GLOBAL_INIT_COMPLETE.load(Ordering::Acquire)
}

#[allow(dead_code)]
fn locate_phdrs() -> &'static [ProgramHeader] {
    unsafe extern "C" {
        static __ehdr_start: ElfHeader;
    }
    unsafe {
        if !__ehdr_start.is_magic_valid() {
            abort!("ELF header magic mismatch")
        }
        __ehdr_start.locate_phdrs()
    }
}

#[cfg(target_arch = "arm")]
#[linkage = "weak"]
#[unsafe(no_mangle)]
extern "C" fn __aeabi_read_tp() -> usize {
    let mut val: usize;
    unsafe {
        core::arch::asm!("mrc p15, 0, {val}, c13, c0, 2", val = out(reg) val); // tpidrurw
    }
    val
}

#[doc(hidden)]
#[allow(unreachable_code)]
pub unsafe fn _run_entrypoint(global_init_cond: bool, f: impl FnOnce()) -> ! {
    unsafe {
        with_local_init(|| {
            if global_init_cond {
                global_init();
            }
            f();
            abort!("entrypoint returned")
        });
    }
}

#[doc(hidden)]
pub mod _private {
    pub use super::_run_entrypoint;
    pub use cfg_if::cfg_if;
    pub use core::arch::global_asm;
    pub use sel4_stack::{Stack, StackBottom};
}
