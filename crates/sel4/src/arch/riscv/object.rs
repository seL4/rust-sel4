use core::ffi::c_uint;

use crate::{ObjectBlueprint, ObjectBlueprintSeL4Arch, ObjectType, ObjectTypeSeL4Arch};

pub type ObjectTypeArch = ObjectTypeRISCV;

pub type ObjectBlueprintArch = ObjectBlueprintRISCV;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ObjectTypeRISCV {
    SeL4Arch(ObjectTypeSeL4Arch),
}

impl ObjectTypeRISCV {
    pub(crate) const fn into_sys(self) -> c_uint {
        match self {
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
    SeL4Arch(ObjectBlueprintSeL4Arch),
}

impl ObjectBlueprintRISCV {
    pub(crate) const fn ty(self) -> ObjectTypeRISCV {
        match self {
            Self::SeL4Arch(sel4_arch) => ObjectTypeRISCV::SeL4Arch(sel4_arch.ty()),
        }
    }

    pub(crate) const fn physical_size_bits(self) -> usize {
        match self {
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
