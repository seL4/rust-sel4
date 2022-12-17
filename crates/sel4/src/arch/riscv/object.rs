use core::ffi::c_uint;

use crate::ObjectType;

pub type ObjectTypeArch = ObjectTypeRISCV;

pub type ObjectBlueprintArch = ObjectBlueprintRISCV;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ObjectTypeRISCV {}

impl ObjectTypeRISCV {
    pub const fn into_sys(self) -> c_uint {
        match self {}
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectBlueprintRISCV {}

impl ObjectBlueprintRISCV {
    pub fn ty(self) -> ObjectType {
        match self {}
    }

    pub fn physical_size_bits(self) -> usize {
        match self {}
    }
}
