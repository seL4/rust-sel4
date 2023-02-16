#[cfg(target_pointer_width = "32")]
pub type Word = u32;

#[cfg(target_pointer_width = "64")]
pub type Word = u64;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ElfHeader {
    pub e_ident: ElfHeaderIdent,
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: Word,
    pub e_phoff: Word,
    pub e_shoff: Word,
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
    pub fn check_magic(&self) -> bool {
        self.e_ident.magic == ELFMAG
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
