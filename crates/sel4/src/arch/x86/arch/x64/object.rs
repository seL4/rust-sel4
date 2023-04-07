use core::ffi::c_uint;

use crate::{sys, ObjectType};

pub type ObjectTypeSeL4Arch = ObjectTypeX64;

pub type ObjectBlueprintSeL4Arch = ObjectBlueprintX64;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ObjectTypeX64 {
    HugePage,
    PDPT,
    PML4,
}

impl ObjectTypeX64 {
    pub const fn into_sys(self) -> c_uint {
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
    pub const fn ty(self) -> ObjectType {
        match self {
            Self::HugePage => ObjectTypeX64::HugePage.into(),
            Self::PDPT => ObjectTypeX64::PDPT.into(),
            Self::PML4 => ObjectTypeX64::PML4.into(),
        }
    }

    pub const fn physical_size_bits(self) -> usize {
        match self {
            Self::HugePage => sys::seL4_HugePageBits.try_into().ok().unwrap(),
            Self::PDPT => sys::seL4_PDPTBits.try_into().ok().unwrap(),
            Self::PML4 => sys::seL4_PML4Bits.try_into().ok().unwrap(),
        }
    }
}
