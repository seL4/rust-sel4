//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(never_type)]
#![feature(cfg_target_thread_local)]

use sel4_elf_header::{ElfHeader, ProgramHeader};
use sel4_panicking_env::abort;

#[cfg(feature = "start")]
mod start;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "tls", target_thread_local))] {
        mod tls;

        #[allow(clippy::missing_safety_doc)]
        pub unsafe fn with_or_without_tls(f: impl FnOnce() -> !) -> ! {
            tls::with_tls(f)
        }
    } else {
        #[allow(clippy::missing_safety_doc)]
        pub unsafe fn with_or_without_tls(f: impl FnOnce() -> !) -> ! {
            f()
        }
    }
}

#[cfg(all(feature = "unwinding", panic = "unwind"))]
mod unwinding;

#[cfg(all(feature = "unwinding", panic = "unwind"))]
pub use self::unwinding::set_eh_frame_finder;

#[allow(dead_code)]
pub(crate) fn locate_phdrs() -> &'static [ProgramHeader] {
    extern "C" {
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
