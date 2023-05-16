use crate::phdrs::elf::{ElfHeader, ProgramHeader};

pub(crate) fn locate_phdrs() -> &'static [ProgramHeader] {
    extern "C" {
        static __ehdr_start: ElfHeader;
    }
    unsafe {
        assert!(__ehdr_start.check_magic());
        __ehdr_start.locate_phdrs()
    }
}
