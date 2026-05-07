//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(linkage)]

use core::error::Error;
use core::fmt;
use core::ops::Range;
use core::slice;

pub use sel4_phdrs_constants::*;

unsafe extern "Rust" {
    safe fn __sel4_phdrs__locate_phdrs() -> Result<ProgramHeaders<'static>, &'static dyn Error>;
}

#[macro_export]
macro_rules! register_locate_phdrs {
    ($(#[$attrs:meta])* $path:path) => {
        #[allow(non_snake_case)]
        const _: () = {
            $(#[$attrs])*
            #[unsafe(no_mangle)]
            fn __sel4_phdrs__locate_phdrs() -> $crate::_private::LocatePhdrsResult {
                const F: fn() -> $crate::_private::LocatePhdrsResult = $path;
                F()
            }
        };
    };
}

register_locate_phdrs!(
    #[linkage = "weak"]
    default_locate_phdrs
);

pub fn locate_phdrs() -> Result<ProgramHeaders<'static>, &'static dyn Error> {
    __sel4_phdrs__locate_phdrs()
}

pub struct ProgramHeaders<'a> {
    slice: &'a [ProgramHeader],
}

impl<'a> ProgramHeaders<'a> {
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn new(start: *const ProgramHeader, n: usize) -> Self {
        Self {
            slice: unsafe { slice::from_raw_parts(start, n) },
        }
    }

    pub fn as_slice(&self) -> &'a [ProgramHeader] {
        self.slice
    }

    pub fn iter(&self) -> impl Iterator<Item = &'a ProgramHeader> {
        self.as_slice().iter()
    }

    pub fn find_by_type(&self, ty: u32) -> Option<&'a ProgramHeader> {
        self.iter().find(|phdr| phdr.p_type == ty)
    }

    pub fn footprint(&self) -> Option<Range<usize>> {
        let start = self
            .iter()
            .filter(|phdr| phdr.p_type == PT_LOAD)
            .map(|phdr| phdr.p_vaddr)
            .min()?;
        let end = self
            .iter()
            .filter(|phdr| phdr.p_type == PT_LOAD)
            .map(|phdr| phdr.p_vaddr + phdr.p_memsz)
            .max()?;
        Some(start..end)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct ElfHeader {
    pub e_ident: ElfHeaderIdent,
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: usize,
    pub e_phoff: usize,
    pub e_shoff: usize,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct ElfHeaderIdent {
    pub magic: [u8; 4],
    pub class: u8,
    pub data: u8,
    pub version: u8,
    pub os_abi: u8,
    pub abi_version: u8,
    pub padding: [u8; 7],
}

const ELFMAG: [u8; 4] = [0x7f, b'E', b'L', b'F'];

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProgramHeader {
    pub p_type: u32,
    #[cfg(target_pointer_width = "64")]
    pub p_flags: u32,
    pub p_offset: usize,
    pub p_vaddr: usize,
    pub p_paddr: usize,
    pub p_filesz: usize,
    pub p_memsz: usize,
    #[cfg(target_pointer_width = "32")]
    pub p_flags: u32,
    pub p_align: usize,
}

pub const PT_NULL: u32 = 0;
pub const PT_LOAD: u32 = 1;
pub const PT_TLS: u32 = 7;
pub const PT_GNU_EH_FRAME: u32 = 0x6474_e550;

impl ProgramHeader {
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn bytes(&self) -> &'static [u8] {
        unsafe { slice::from_raw_parts(self.p_vaddr as *const u8, self.p_memsz) }
    }
}

pub fn default_locate_phdrs() -> Result<ProgramHeaders<'static>, &'static dyn Error> {
    unsafe extern "C" {
        safe static __ehdr_start: ElfHeader;
    }
    if __ehdr_start.e_ident.magic != ELFMAG {
        return Err(&DefaultLocatePhdrsError::InvalidMagic);
    }
    if __ehdr_start.e_phoff != size_of::<ElfHeader>() {
        return Err(&DefaultLocatePhdrsError::UnexpectedPhoff);
    }
    let start = (&raw const __ehdr_start)
        .wrapping_byte_add(__ehdr_start.e_phoff)
        .cast();
    Ok(unsafe { ProgramHeaders::new(start, __ehdr_start.e_phnum.into()) })
}

#[derive(Debug, Copy, Clone)]
pub enum DefaultLocatePhdrsError {
    InvalidMagic,
    UnexpectedPhoff,
}

impl fmt::Display for DefaultLocatePhdrsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic => write!(f, "invalid magic in ELF header"),
            Self::UnexpectedPhoff => write!(f, "unexpected e_phoff in ELF header"),
        }
    }
}

impl Error for DefaultLocatePhdrsError {}

// For macros
#[doc(hidden)]
pub mod _private {
    use core::error::Error;

    use super::ProgramHeaders;

    pub type LocatePhdrsResult = Result<ProgramHeaders<'static>, &'static dyn Error>;
}
