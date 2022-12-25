use core::ffi::c_uint;

use crate::ObjectType;

pub type ObjectTypeSeL4Arch = ObjectTypeRISCV64;

pub type ObjectBlueprintSeL4Arch = ObjectBlueprintRISCV64;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ObjectTypeRISCV64 {}

impl ObjectTypeRISCV64 {
    pub const fn into_sys(self) -> c_uint {
        match self {}
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectBlueprintRISCV64 {}

impl ObjectBlueprintRISCV64 {
    pub const fn ty(self) -> ObjectType {
        match self {}
    }

    pub const fn physical_size_bits(self) -> usize {
        match self {}
    }
}
