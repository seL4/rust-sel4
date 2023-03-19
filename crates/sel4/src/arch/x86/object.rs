use core::ffi::c_uint;

use crate::{sys, ObjectBlueprint, ObjectBlueprintSeL4Arch, ObjectType, ObjectTypeSeL4Arch};

pub type ObjectTypeArch = ObjectTypeX86;

pub type ObjectBlueprintArch = ObjectBlueprintX86;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ObjectTypeX86 {
    _4K,
    LargePage,
    SeL4Arch(ObjectTypeSeL4Arch),
}

impl ObjectTypeX86 {
    pub const fn into_sys(self) -> c_uint {
        match self {
            Self::_4K => sys::_object::seL4_X86_4K,
            Self::LargePage => sys::_object::seL4_X86_LargePageObject,
            Self::SeL4Arch(sel4_arch) => sel4_arch.into_sys(),
        }
    }
}

impl const From<ObjectTypeSeL4Arch> for ObjectTypeArch {
    fn from(ty: ObjectTypeSeL4Arch) -> Self {
        Self::SeL4Arch(ty)
    }
}

impl const From<ObjectTypeSeL4Arch> for ObjectType {
    fn from(ty: ObjectTypeSeL4Arch) -> Self {
        Self::from(ObjectTypeArch::from(ty))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectBlueprintX86 {
    _4K,
    LargePage,
    SeL4Arch(ObjectBlueprintSeL4Arch),
}

impl ObjectBlueprintX86 {
    pub const fn ty(self) -> ObjectType {
        match self {
            Self::_4K => ObjectTypeX86::_4K.into(),
            Self::LargePage => ObjectTypeX86::LargePage.into(),
            Self::SeL4Arch(sel4_arch) => sel4_arch.ty(),
        }
    }

    pub const fn physical_size_bits(self) -> usize {
        match self {
            Self::_4K => sys::seL4_PageBits.try_into().ok().unwrap(),
            Self::LargePage => sys::seL4_LargePageBits.try_into().ok().unwrap(),
            Self::SeL4Arch(sel4_arch) => sel4_arch.physical_size_bits(),
        }
    }
}

impl const From<ObjectBlueprintSeL4Arch> for ObjectBlueprintArch {
    fn from(blueprint: ObjectBlueprintSeL4Arch) -> Self {
        Self::SeL4Arch(blueprint)
    }
}

impl const From<ObjectBlueprintSeL4Arch> for ObjectBlueprint {
    fn from(ty: ObjectBlueprintSeL4Arch) -> Self {
        Self::from(ObjectBlueprintArch::from(ty))
    }
}
