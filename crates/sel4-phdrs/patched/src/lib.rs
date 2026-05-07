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

register_locate_phdrs!(locate_patched_phdrs);

pub fn locate_patched_phdrs() -> Result<ProgramHeaders<'static>, &'static dyn Error> {
    let start = *rodata_static!(sel4_phdrs_patched__vaddr: *const ProgramHeader);
    let phnum = *rodata_static!(sel4_phdrs_patched__phnum: u16);
    if phnum == 0 {
        return Err(&LocatePatchedPhdrsError::PhnumIsZero);
    }
    Ok(unsafe { ProgramHeaders::new(start, phnum.into()) })
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
