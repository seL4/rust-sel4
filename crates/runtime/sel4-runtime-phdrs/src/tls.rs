use sel4_reserve_tls_on_stack::TlsImage;

use crate::elf::{ProgramHeader, Word, PT_TLS};

impl TryFrom<&ProgramHeader> for TlsImage {
    type Error = <usize as TryFrom<Word>>::Error;

    fn try_from(phdr: &ProgramHeader) -> Result<Self, Self::Error> {
        assert_eq!(phdr.p_type, PT_TLS);
        Ok(Self {
            vaddr: phdr.p_vaddr.try_into()?,
            filesz: phdr.p_filesz.try_into()?,
            memsz: phdr.p_memsz.try_into()?,
            align: phdr.p_align.try_into()?,
        })
    }
}
