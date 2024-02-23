//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_elf_header::{ElfHeader, ProgramHeader};
use sel4_panicking_env::abort;

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
