use core::ffi::c_uint;

use crate::ObjectType;

pub type ObjectTypeArch = ObjectTypeX86;

pub type ObjectBlueprintArch = ObjectBlueprintX86;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ObjectTypeX86 {}

impl ObjectTypeX86 {
    pub const fn into_sys(self) -> c_uint {
        match self {}
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectBlueprintX86 {}

impl ObjectBlueprintX86 {
    pub fn ty(self) -> ObjectType {
        match self {}
    }

    pub fn physical_size_bits(self) -> usize {
        match self {}
    }
}
