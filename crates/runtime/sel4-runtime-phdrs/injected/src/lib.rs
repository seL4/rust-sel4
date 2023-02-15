#![no_std]
#![feature(cfg_target_thread_local)]

use core::mem;

use sel4_runtime_phdrs::{elf::ProgramHeader, InnerProgramHeadersFinder, ProgramHeadersFinder};

const MAX_NUM_PHDRS: usize = 16;

// HACK until core::mem::zeroed is const
const PLACEHOLDER_PHDR: ProgramHeader =
    unsafe { mem::transmute([0u8; mem::size_of::<ProgramHeader>()]) };

#[no_mangle]
#[link_section = ".data"]
static mut __num_phdrs: usize = 0;

#[no_mangle]
#[link_section = ".data"]
static mut __phdrs: [ProgramHeader; MAX_NUM_PHDRS] = [PLACEHOLDER_PHDR; MAX_NUM_PHDRS];

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InjectedProgramHeaders(());

impl InjectedProgramHeaders {
    pub const fn new() -> Self {
        Self(())
    }

    pub const fn finder() -> ProgramHeadersFinder<Self> {
        ProgramHeadersFinder::new(Self::new())
    }
}

impl InnerProgramHeadersFinder for InjectedProgramHeaders {
    fn find_phdrs(&self) -> &[ProgramHeader] {
        let phdrs = unsafe { &__phdrs[..] };
        let num_phdrs = unsafe { __num_phdrs };
        assert_ne!(num_phdrs, 0);
        &phdrs[..num_phdrs]
    }
}
