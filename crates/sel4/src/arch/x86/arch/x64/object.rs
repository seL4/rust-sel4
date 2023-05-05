use core::ffi::c_uint;

use crate::{const_helpers::u32_into_usize, sys};

pub type ObjectTypeSeL4Arch = ObjectTypeX64;

pub type ObjectBlueprintSeL4Arch = ObjectBlueprintX64;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ObjectTypeX64 {
    HugePage,
    PDPT,
    PML4,
}

impl ObjectTypeX64 {
    pub(crate) const fn into_sys(self) -> c_uint {
        match self {
            Self::HugePage => sys::_mode_object::seL4_X64_HugePageObject,
            Self::PDPT => sys::_mode_object::seL4_X86_PDPTObject,
            Self::PML4 => sys::_mode_object::seL4_X64_PML4Object,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectBlueprintX64 {
    HugePage,
    PDPT,
    PML4,
}

impl ObjectBlueprintX64 {
    pub(crate) const fn ty(self) -> ObjectTypeX64 {
        match self {
            Self::HugePage => ObjectTypeX64::HugePage,
            Self::PDPT => ObjectTypeX64::PDPT,
            Self::PML4 => ObjectTypeX64::PML4,
        }
    }

    pub(crate) const fn physical_size_bits(self) -> usize {
        match self {
            Self::HugePage => u32_into_usize(sys::seL4_HugePageBits),
            Self::PDPT => u32_into_usize(sys::seL4_PDPTBits),
            Self::PML4 => u32_into_usize(sys::seL4_PML4Bits),
        }
    }
}
