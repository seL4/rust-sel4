#![no_std]

use core::mem;

use sel4_runtime_building_blocks_elf::ProgramHeader;

const MAX_NUM_PHDRS: usize = 16;

// [HACK]
// Until zerocopy::new_zeroed works in const.
// [SAFETY]
// Safe because ProgramHeader implements FromBytes.
const PLACEHOLDER_PHDR: ProgramHeader =
    unsafe { mem::transmute([0u8; mem::size_of::<ProgramHeader>()]) };

#[no_mangle]
#[link_section = ".data"]
static mut __num_phdrs: usize = 0;

#[no_mangle]
#[link_section = ".data"]
static mut __phdrs: [ProgramHeader; MAX_NUM_PHDRS] = [PLACEHOLDER_PHDR; MAX_NUM_PHDRS];

pub fn get_phdrs() -> &'static [ProgramHeader] {
    let phdrs = unsafe { &__phdrs[..] };
    let num_phdrs = unsafe { __num_phdrs };
    assert_ne!(num_phdrs, 0);
    &phdrs[..num_phdrs]
}
