#![no_std]

use core::mem;

use sel4_runtime_building_blocks_elf::ProgramHeader;

const MAX_NUM_PHDRS: usize = 16;

const PLACEHOLDER_NUM_PHDRS: usize = usize::MAX;

// [HACK]
// Until zerocopy::transmute works in const.
// [NOTE]
// Initialize to != zero to avoid having to use e.g. section attributes to avoid these ending up in .bss.
// [SAFETY]
// Safe because ProgramHeader implements FromBytes.
const PLACEHOLDER_PHDR: ProgramHeader =
    unsafe { mem::transmute([0xffu8; mem::size_of::<ProgramHeader>()]) };

#[used]
#[no_mangle]
static mut __num_phdrs: usize = PLACEHOLDER_NUM_PHDRS;

#[used]
#[no_mangle]
static mut __phdrs: [ProgramHeader; MAX_NUM_PHDRS] = [PLACEHOLDER_PHDR; MAX_NUM_PHDRS];

pub fn get_phdrs() -> &'static [ProgramHeader] {
    let phdrs = unsafe { &__phdrs[..] };
    let num_phdrs = unsafe { __num_phdrs };
    assert_ne!(num_phdrs, PLACEHOLDER_NUM_PHDRS);
    &phdrs[..num_phdrs]
}
