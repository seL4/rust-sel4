use core::ffi::c_uint;

use crate::{
    const_helpers::u32_into_usize, sys, ObjectBlueprint, ObjectBlueprintSeL4Arch, ObjectType,
    ObjectTypeSeL4Arch,
};

pub type ObjectTypeArch = ObjectTypeX86;

pub type ObjectBlueprintArch = ObjectBlueprintX86;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ObjectTypeX86 {
    _4K,
    LargePage,
    PageTable,
    PageDirectory,
    SeL4Arch(ObjectTypeSeL4Arch),
}

impl ObjectTypeX86 {
    pub(crate) const fn into_sys(self) -> c_uint {
        match self {
            Self::_4K => sys::_object::seL4_X86_4K,
            Self::LargePage => sys::_object::seL4_X86_LargePageObject,
            Self::PageTable => sys::_object::seL4_X86_PageTableObject,
            Self::PageDirectory => sys::_object::seL4_X86_PageDirectoryObject,
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
pub enum ObjectBlueprintX86 {
    _4K,
    LargePage,
    PageTable,
    PageDirectory,
    SeL4Arch(ObjectBlueprintSeL4Arch),
}

impl ObjectBlueprintX86 {
    pub(crate) const fn ty(self) -> ObjectTypeX86 {
        match self {
            Self::_4K => ObjectTypeX86::_4K,
            Self::LargePage => ObjectTypeX86::LargePage,
            Self::PageTable => ObjectTypeX86::PageTable,
            Self::PageDirectory => ObjectTypeX86::PageDirectory,
            Self::SeL4Arch(sel4_arch) => ObjectTypeX86::SeL4Arch(sel4_arch.ty()),
        }
    }

    pub(crate) const fn physical_size_bits(self) -> usize {
        match self {
            Self::_4K => u32_into_usize(sys::seL4_PageBits),
            Self::LargePage => u32_into_usize(sys::seL4_LargePageBits),
            Self::PageTable => u32_into_usize(sys::seL4_PageTableBits),
            Self::PageDirectory => u32_into_usize(sys::seL4_PageDirBits),
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
