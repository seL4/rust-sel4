//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(cfg_target_thread_local)]
#![feature(core_intrinsics)]
#![feature(never_type)]
#![allow(internal_features)]

use sel4_elf_header::{ElfHeader, ProgramHeader};
use sel4_panicking_env::abort;

#[cfg(target_thread_local)]
mod tls;

#[cfg(panic = "unwind")]
mod unwinding;

#[cfg(feature = "abort")]
mod abort;

#[cfg(feature = "start")]
mod start;

#[allow(clippy::missing_safety_doc)]
pub unsafe fn with_local_initialization(f: impl FnOnce() -> !) -> ! {
    cfg_if::cfg_if! {
        if #[cfg(target_thread_local)] {
            tls::with_tls(f)
        } else {
            f()
        }
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn global_initialzation() {
    #[cfg(panic = "unwind")]
    {
        crate::unwinding::set_eh_frame_finder().unwrap();
    }

    sel4_ctors_dtors::run_ctors().unwrap();
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

#[doc(hidden)]
pub mod _private {
    #[cfg(feature = "start")]
    pub use crate::start::_private as start;
}
