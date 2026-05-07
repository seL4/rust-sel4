//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::error::Error;
use core::fmt;

use sel4_phdrs::{ProgramHeader, ProgramHeaders, register_locate_phdrs};
use sel4_rodata_static::rodata_static;

// register_locate_phdrs!(locate_patched_phdrs);

pub fn locate_patched_phdrs() -> Result<ProgramHeaders<'static>, &'static dyn Error> {
    let start = *rodata_static!(sel4_phdrs_patched__start: *const ProgramHeader);
    let phnum = *rodata_static!(sel4_phdrs_patched__phnum: usize);
    if phnum == 0 {
        return Err(&LocatePatchedPhdrsError::PhnumIsZero);
    }
    Ok(unsafe { ProgramHeaders::new(start, phnum) })
}

#[derive(Debug, Copy, Clone)]
pub enum LocatePatchedPhdrsError {
    PhnumIsZero,
}

impl fmt::Display for LocatePatchedPhdrsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PhnumIsZero => write!(f, "patched phdr count is zero"),
        }
    }
}

impl Error for LocatePatchedPhdrsError {}

#[macro_export]
macro_rules! add_placeholder_phdrs {
    ($label:ident) => {
        const _: () = {
            #[unsafe(link_section = $crate::_private::concat!(".note.sel4.placeholder.", $crate::_private::stringify!($label), ".1"))]
            static _1: [u32; 0] = [];

            #[unsafe(link_section = $crate::_private::concat!(".note.sel4.placeholder.", $crate::_private::stringify!($label), ".2"))]
            static _2: [u64; 0] = [];

            #[unsafe(link_section = $crate::_private::concat!(".note.sel4.placeholder.", $crate::_private::stringify!($label), ".3"))]
            static _3: [u32; 0] = [];

        };
    };
}

// For macros
#[doc(hidden)]
pub mod _private {
    pub use core::{concat, stringify};
}
