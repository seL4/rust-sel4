use core::ffi::c_uint;

use crate::{sys, ObjectType};

pub type ObjectTypeSeL4Arch = ObjectTypeX64;

pub type ObjectBlueprintSeL4Arch = ObjectBlueprintX64;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ObjectTypeX64 {
    HugePage,
}

impl ObjectTypeX64 {
    pub const fn into_sys(self) -> c_uint {
        match self {
            Self::HugePage => sys::_mode_object::seL4_X64_HugePageObject,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectBlueprintX64 {
    HugePage,
}

impl ObjectBlueprintX64 {
    pub const fn ty(self) -> ObjectType {
        match self {
            Self::HugePage => ObjectTypeX64::HugePage.into(),
        }
    }

    pub const fn physical_size_bits(self) -> usize {
        match self {
            Self::HugePage => sys::seL4_HugePageBits.try_into().ok().unwrap(),
        }
    }
}
