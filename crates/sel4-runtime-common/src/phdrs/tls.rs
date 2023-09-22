use sel4_initialize_tls_on_stack::TlsImage;

use crate::phdrs::{elf::PT_TLS, locate_phdrs};

pub fn locate_tls_image() -> Option<TlsImage> {
    locate_phdrs()
        .iter()
        .find(|phdr| phdr.p_type == PT_TLS)
        .map(|phdr| TlsImage {
            vaddr: phdr.p_vaddr.try_into().unwrap(),
            filesz: phdr.p_filesz.try_into().unwrap(),
            memsz: phdr.p_memsz.try_into().unwrap(),
            align: phdr.p_align.try_into().unwrap(),
        })
}
