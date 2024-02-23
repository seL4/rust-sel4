//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(cfg_target_thread_local)]

use sel4_elf_header::{ElfHeader, ProgramHeader};
use sel4_panicking_env::abort;

mod ctors;

pub use ctors::run_ctors;

#[cfg(feature = "start")]
mod start;

#[cfg(all(feature = "tls", target_thread_local))]
mod tls;

#[cfg(all(feature = "tls", target_thread_local))]
pub use tls::{initialize_tls_on_stack_and_continue, ContArg, ContFn};

#[cfg(all(feature = "unwinding", panic = "unwind"))]
mod unwinding;

#[cfg(all(feature = "unwinding", panic = "unwind"))]
pub use self::unwinding::set_eh_frame_finder;

pub(crate) fn locate_phdrs() -> &'static [ProgramHeader] {
    extern "C" {
        static __ehdr_start: ElfHeader;
    }
    unsafe {
        if !__ehdr_start.check_magic() {
            abort!("ELF header magic mismatch")
        }
        __ehdr_start.locate_phdrs()
    }
}

#[cfg(target_arch = "arm")]
core::arch::global_asm! {
    r#"
        .global __aeabi_read_tp

        .section .text

        __aeabi_read_tp:
            mrc p15, 0, r0, c13, c0, 2
            bx lr
    "#
}

#[doc(hidden)]
pub mod _private {
    #[cfg(feature = "start")]
    pub use crate::start::_private as start;
}
