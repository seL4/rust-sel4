//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_panicking_env::abort;

mod elf;

use elf::{ElfHeader, ProgramHeader};

#[cfg(all(feature = "tls", target_thread_local))]
mod tls;

#[cfg(all(feature = "tls", target_thread_local))]
pub use tls::{initialize_tls_on_stack_and_continue, ContArg, ContFn};

#[cfg(feature = "unwinding")]
mod unwinding;

#[cfg(feature = "unwinding")]
pub use self::unwinding::set_eh_frame_finder;

pub(crate) fn locate_phdrs() -> &'static [ProgramHeader] {
    extern "C" {
        static __ehdr_start: ElfHeader;
    }
    unsafe {
        if !__ehdr_start.check_magic() {
            abort!()
        }
        __ehdr_start.locate_phdrs()
    }
}
