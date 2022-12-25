use core::ffi::c_uint;

use crate::{ObjectBlueprint, ObjectBlueprintSeL4Arch, ObjectType, ObjectTypeSeL4Arch};

pub type ObjectTypeArch = ObjectTypeX86;

pub type ObjectBlueprintArch = ObjectBlueprintX86;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ObjectTypeX86 {
    SeL4Arch(ObjectTypeSeL4Arch),
}

impl ObjectTypeX86 {
    pub const fn into_sys(self) -> c_uint {
        match self {
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
    SeL4Arch(ObjectBlueprintSeL4Arch),
}

impl ObjectBlueprintX86 {
    pub const fn ty(self) -> ObjectType {
        match self {
            Self::SeL4Arch(sel4_arch) => sel4_arch.ty(),
        }
    }

    pub const fn physical_size_bits(self) -> usize {
        match self {
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
