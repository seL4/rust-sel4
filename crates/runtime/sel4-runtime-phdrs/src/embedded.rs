use core::slice;

use crate::{elf::*, InnerProgramHeadersFinder, ProgramHeadersFinder};

extern "C" {
    static __ehdr_start: ElfHeader;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EmbeddedProgramHeaders(());

impl EmbeddedProgramHeaders {
    pub const fn new() -> Self {
        Self(())
    }

    pub const fn finder() -> ProgramHeadersFinder<Self> {
        ProgramHeadersFinder::new(Self::new())
    }
}

impl InnerProgramHeadersFinder for EmbeddedProgramHeaders {
    fn find_phdrs(&self) -> &[ProgramHeader] {
        unsafe {
            assert!(__ehdr_start.check_magic());
            let ptr = (&__ehdr_start as *const ElfHeader)
                .cast::<u8>()
                .offset(__ehdr_start.e_phoff.try_into().unwrap())
                .cast::<ProgramHeader>();
            slice::from_raw_parts(ptr, __ehdr_start.e_phnum.try_into().unwrap())
        }
    }
}
