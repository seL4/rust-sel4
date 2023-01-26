#![no_std]

use sel4_runtime_building_blocks_elf::*;

extern "C" {
    static __ehdr_start: ElfHeader;
}

pub fn get_phdrs() -> &'static [ProgramHeader] {
    unsafe {
        assert!(__ehdr_start.check_magic());
        let ptr = (&__ehdr_start as *const ElfHeader)
            .cast::<u8>()
            .offset(__ehdr_start.e_phoff.try_into().unwrap())
            .cast::<ProgramHeader>();
        core::slice::from_raw_parts(ptr, __ehdr_start.e_phnum.try_into().unwrap())
    }
}
