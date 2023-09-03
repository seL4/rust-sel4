use core::ffi::c_uint;

use crate::{
    const_helpers::u32_into_usize, sys, ObjectBlueprint, ObjectBlueprintSeL4Arch, ObjectType,
    ObjectTypeSeL4Arch,
};

pub type ObjectTypeArch = ObjectTypeRISCV;

pub type ObjectBlueprintArch = ObjectBlueprintRISCV;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ObjectTypeRISCV {
    _4KPage,
    MegaPage,
    PageTable,
    SeL4Arch(ObjectTypeSeL4Arch),
}

impl ObjectTypeRISCV {
    pub(crate) const fn into_sys(self) -> c_uint {
        match self {
            Self::_4KPage => sys::_object::seL4_RISCV_4K_Page,
            Self::MegaPage => sys::_object::seL4_RISCV_Mega_Page,
            Self::PageTable => sys::_object::seL4_RISCV_PageTableObject,
            Self::SeL4Arch(sel4_arch) => sel4_arch.into_sys(),
        }
    }
}

impl From<ObjectTypeSeL4Arch> for ObjectTypeArch {
    fn from(ty: ObjectTypeSeL4Arch) -> Self {
        Self::SeL4Arch(ty)
    }
}

impl From<ObjectTypeSeL4Arch> for ObjectType {
    fn from(ty: ObjectTypeSeL4Arch) -> Self {
        Self::from(ObjectTypeArch::from(ty))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectBlueprintRISCV {
    _4KPage,
    MegaPage,
    PageTable,
    SeL4Arch(ObjectBlueprintSeL4Arch),
}

impl ObjectBlueprintRISCV {
    pub(crate) const fn ty(self) -> ObjectTypeRISCV {
        match self {
            Self::_4KPage => ObjectTypeRISCV::_4KPage,
            Self::MegaPage => ObjectTypeRISCV::MegaPage,
            Self::PageTable => ObjectTypeRISCV::PageTable,
            Self::SeL4Arch(sel4_arch) => ObjectTypeRISCV::SeL4Arch(sel4_arch.ty()),
        }
    }

    pub(crate) const fn physical_size_bits(self) -> usize {
        match self {
            Self::_4KPage => u32_into_usize(sys::seL4_PageBits),
            Self::MegaPage => u32_into_usize(sys::seL4_LargePageBits),
            Self::PageTable => u32_into_usize(sys::seL4_PageTableBits),
            Self::SeL4Arch(sel4_arch) => sel4_arch.physical_size_bits(),
        }
    }
}

impl From<ObjectBlueprintSeL4Arch> for ObjectBlueprintArch {
    fn from(blueprint: ObjectBlueprintSeL4Arch) -> Self {
        Self::SeL4Arch(blueprint)
    }
}

impl From<ObjectBlueprintSeL4Arch> for ObjectBlueprint {
    fn from(ty: ObjectBlueprintSeL4Arch) -> Self {
        Self::from(ObjectBlueprintArch::from(ty))
    }
}
