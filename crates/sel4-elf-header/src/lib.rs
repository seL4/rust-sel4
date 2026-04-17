//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::error::Error;
use core::fmt;
use core::ops::Range;
use core::ptr;
use core::slice;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ElfHeader {
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
pub struct ElfHeaderIdent {
    pub magic: [u8; 4],
    pub class: u8,
    pub data: u8,
    pub version: u8,
    pub os_abi: u8,
    pub abi_version: u8,
    pub padding: [u8; 7],
}

pub const ELFMAG: [u8; 4] = [0x7f, b'E', b'L', b'F'];

impl ElfHeader {
    pub fn is_magic_valid(&self) -> bool {
        self.e_ident.magic == ELFMAG
    }

    pub fn check_magic(&self) -> Result<(), InvalidMagic> {
        if self.is_magic_valid() {
            Ok(())
        } else {
            Err(InvalidMagic {
                ehdr_addr: self as *const _ as usize,
            })
        }
    }

    pub fn locate_phdrs(&'static self) -> &'static [ProgramHeader] {
        unsafe {
            let ptr = ptr::from_ref(self)
                .cast::<u8>()
                .wrapping_byte_offset(self.e_phoff as isize)
                .cast::<ProgramHeader>();
            slice::from_raw_parts(ptr, self.e_phnum.into())
        }
    }
}

pub fn locate_phdrs() -> Result<&'static [ProgramHeader], InvalidMagic> {
    unsafe extern "C" {
        static __ehdr_start: ElfHeader;
    }
    unsafe {
        __ehdr_start.check_magic()?;
    }
    Ok(unsafe { __ehdr_start.locate_phdrs() })
}

#[derive(Debug, Copy, Clone)]
pub struct InvalidMagic {
    ehdr_addr: usize,
}

impl fmt::Display for InvalidMagic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid magic for ELF header at 0x{:x}", self.ehdr_addr)
    }
}

impl Error for InvalidMagic {}

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
    pub fn vaddr_range(&self) -> Range<usize> {
        let start = self.p_vaddr;
        let end = start + self.p_memsz;
        start..end
    }
}
