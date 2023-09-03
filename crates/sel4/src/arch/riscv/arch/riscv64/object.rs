use core::ffi::c_uint;

use crate::{const_helpers::u32_into_usize, sys};

pub type ObjectTypeSeL4Arch = ObjectTypeRISCV64;

pub type ObjectBlueprintSeL4Arch = ObjectBlueprintRISCV64;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ObjectTypeRISCV64 {
    GigaPage,
}

impl ObjectTypeRISCV64 {
    pub(crate) const fn into_sys(self) -> c_uint {
        match self {
            Self::GigaPage => sys::_mode_object::seL4_RISCV_Giga_Page,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectBlueprintRISCV64 {
    GigaPage,
}

impl ObjectBlueprintRISCV64 {
    pub(crate) const fn ty(self) -> ObjectTypeRISCV64 {
        match self {
            Self::GigaPage => ObjectTypeRISCV64::GigaPage,
        }
    }

    pub(crate) const fn physical_size_bits(self) -> usize {
        match self {
            Self::GigaPage => u32_into_usize(sys::seL4_HugePageBits),
        }
    }
}
