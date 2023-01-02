#![no_std]

use zerocopy::{AsBytes, FromBytes};

#[cfg(target_pointer_width = "64")]
pub type Word = u64;

#[cfg(target_pointer_width = "32")]
pub type Word = u32;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, AsBytes, FromBytes, Default)]
pub struct ProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: Word,
    pub p_vaddr: Word,
    pub p_paddr: Word,
    pub p_filesz: Word,
    pub p_memsz: Word,
    pub p_align: Word,
}

pub const PT_NULL: u32 = 0;
pub const PT_LOAD: u32 = 1;
pub const PT_TLS: u32 = 7;
pub const PT_GNU_EH_FRAME: u32 = 0x6474_e550;

pub const PF_X: u32 = 1 << 0;
pub const PF_W: u32 = 1 << 1;
pub const PF_R: u32 = 1 << 2;
