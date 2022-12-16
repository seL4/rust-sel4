use core::ffi::c_uint;

use crate::{sys, ObjectType};

pub type ObjectTypeSeL4Arch = ObjectTypeAArch64;

pub type ObjectBlueprintSeL4Arch = ObjectBlueprintAArch64;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ObjectTypeAArch64 {
    HugePage,
    PUD,
    PGD,
}

impl ObjectTypeAArch64 {
    pub const fn into_sys(self) -> c_uint {
        match self {
            Self::HugePage => sys::_mode_object::seL4_ARM_HugePageObject,
            Self::PUD => sys::_mode_object::seL4_ARM_PageUpperDirectoryObject,
            Self::PGD => sys::_mode_object::seL4_ARM_PageGlobalDirectoryObject,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectBlueprintAArch64 {
    HugePage,
    PUD,
    PGD,
}

impl ObjectBlueprintAArch64 {
    pub fn ty(self) -> ObjectType {
        match self {
            Self::HugePage => ObjectTypeAArch64::HugePage.into(),
            Self::PUD => ObjectTypeAArch64::PUD.into(),
            Self::PGD => ObjectTypeAArch64::PGD.into(),
        }
    }

    pub fn physical_size_bits(self) -> usize {
        match self {
            Self::HugePage => sys::seL4_HugePageBits.try_into().unwrap(),
            Self::PUD => sys::seL4_PUDBits.try_into().unwrap(),
            Self::PGD => sys::seL4_PGDBits.try_into().unwrap(),
        }
    }
}
