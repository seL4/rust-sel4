use core::ffi::c_uint;

use sel4_config::{sel4_cfg_enum, sel4_cfg_match};

use crate::{
    const_helpers::u32_into_usize, sys, ObjectBlueprint, ObjectBlueprintSeL4Arch, ObjectType,
    ObjectTypeSeL4Arch,
};

/// Alias for [`ObjectTypeArm`].
pub type ObjectTypeArch = ObjectTypeArm;

/// Alias for [`ObjectBlueprintArm`].
pub type ObjectBlueprintArch = ObjectBlueprintArm;

/// Corresponds to `seL4_ArchObjectType`.
#[sel4_cfg_enum]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ObjectTypeArm {
    SmallPage,
    LargePage,
    PT,
    PD,
    #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
    VCPU,
    SeL4Arch(ObjectTypeSeL4Arch),
}

impl ObjectTypeArm {
    pub(crate) const fn into_sys(self) -> c_uint {
        #[sel4_cfg_match]
        match self {
            Self::SmallPage => sys::_object::seL4_ARM_SmallPageObject,
            Self::LargePage => sys::_object::seL4_ARM_LargePageObject,
            Self::PT => sys::_object::seL4_ARM_PageTableObject,
            Self::PD => sys::_object::seL4_ARM_PageDirectoryObject,
            #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
            Self::VCPU => sys::_object::seL4_ARM_VCPUObject,
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

/// Arm-specific variants of [`ObjectBlueprint`].
#[sel4_cfg_enum]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectBlueprintArm {
    SmallPage,
    LargePage,
    PT,
    PD,
    #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
    VCPU,
    SeL4Arch(ObjectBlueprintSeL4Arch),
}

impl ObjectBlueprintArm {
    pub(crate) const fn ty(self) -> ObjectTypeArch {
        #[sel4_cfg_match]
        match self {
            Self::SmallPage => ObjectTypeArm::SmallPage,
            Self::LargePage => ObjectTypeArm::LargePage,
            Self::PT => ObjectTypeArm::PT,
            Self::PD => ObjectTypeArm::PD,
            #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
            Self::VCPU => ObjectTypeArm::VCPU,
            Self::SeL4Arch(sel4_arch) => ObjectTypeArch::SeL4Arch(sel4_arch.ty()),
        }
    }

    pub(crate) const fn physical_size_bits(self) -> usize {
        #[sel4_cfg_match]
        match self {
            Self::SmallPage => u32_into_usize(sys::seL4_PageBits),
            Self::LargePage => u32_into_usize(sys::seL4_LargePageBits),
            Self::PT => u32_into_usize(sys::seL4_PageTableBits),
            Self::PD => u32_into_usize(sys::seL4_PageDirBits),
            #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
            Self::VCPU => u32_into_usize(sys::seL4_VCPUBits),
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
