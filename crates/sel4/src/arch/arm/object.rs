use core::ffi::c_uint;

use sel4_config::{sel4_cfg_enum, sel4_cfg_match};

use crate::{sys, ObjectBlueprint, ObjectBlueprintSeL4Arch, ObjectType, ObjectTypeSeL4Arch};

/// Alias for [`ObjectTypeArm`].
pub type ObjectTypeArch = ObjectTypeArm;

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
    pub const fn into_sys(self) -> c_uint {
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
    pub const fn ty(self) -> ObjectType {
        #[sel4_cfg_match]
        match self {
            Self::SmallPage => ObjectTypeArm::SmallPage.into(),
            Self::LargePage => ObjectTypeArm::LargePage.into(),
            Self::PT => ObjectTypeArm::PT.into(),
            Self::PD => ObjectTypeArm::PD.into(),
            #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
            Self::VCPU => ObjectTypeArm::VCPU.into(),
            Self::SeL4Arch(sel4_arch) => sel4_arch.ty(),
        }
    }

    pub const fn physical_size_bits(self) -> usize {
        #[sel4_cfg_match]
        match self {
            Self::SmallPage => sys::seL4_PageBits.try_into().ok().unwrap(),
            Self::LargePage => sys::seL4_LargePageBits.try_into().ok().unwrap(),
            Self::PT => sys::seL4_PageTableBits.try_into().ok().unwrap(),
            Self::PD => sys::seL4_PageDirBits.try_into().ok().unwrap(),
            #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
            Self::VCPU => sys::seL4_VCPUBits.try_into().ok().unwrap(),
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
