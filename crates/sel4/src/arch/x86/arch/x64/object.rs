use core::ffi::c_uint;

use crate::ObjectType;

pub type ObjectTypeSeL4Arch = ObjectTypeX64;

pub type ObjectBlueprintSeL4Arch = ObjectBlueprintX64;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ObjectTypeX64 {}

impl ObjectTypeX64 {
    pub const fn into_sys(self) -> c_uint {
        match self {}
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectBlueprintX64 {}

impl ObjectBlueprintX64 {
    pub fn ty(self) -> ObjectType {
        match self {}
    }

    pub fn physical_size_bits(self) -> usize {
        match self {}
    }
}
