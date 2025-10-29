//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use core::ffi::c_uint;

use sel4_config::{sel4_cfg_enum, sel4_cfg_wrap_match};

use crate::{
    ObjectBlueprint, ObjectBlueprintSeL4Arch, ObjectType, ObjectTypeSeL4Arch,
    const_helpers::u32_into_usize, sys,
};

pub type ObjectTypeArch = ObjectTypeX86;

pub type ObjectBlueprintArch = ObjectBlueprintX86;

#[sel4_cfg_enum]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ObjectTypeX86 {
    _4k,
    LargePage,
    PageTable,
    PageDirectory,
    #[sel4_cfg(VTX)]
    VCpu,
    #[sel4_cfg(VTX)]
    EPTPageDirectory,
    #[sel4_cfg(VTX)]
    EPTPageTable,
    SeL4Arch(ObjectTypeSeL4Arch),
}

impl ObjectTypeX86 {
    pub(crate) const fn into_sys(self) -> c_uint {
        sel4_cfg_wrap_match! {
            match self {
                Self::_4k => sys::_object::seL4_X86_4K,
                Self::LargePage => sys::_object::seL4_X86_LargePageObject,
                Self::PageTable => sys::_object::seL4_X86_PageTableObject,
                Self::PageDirectory => sys::_object::seL4_X86_PageDirectoryObject,
                #[sel4_cfg(VTX)]
                Self::VCpu => sys::_object::seL4_X86_VCPUObject,
                #[sel4_cfg(VTX)]
                Self::EPTPageDirectory => sys::_object::seL4_X86_EPTPDObject,
                #[sel4_cfg(VTX)]
                Self::EPTPageTable => sys::_object::seL4_X86_EPTPTObject,
                Self::SeL4Arch(sel4_arch) => sel4_arch.into_sys(),
            }
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

#[sel4_cfg_enum]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectBlueprintX86 {
    _4k,
    LargePage,
    PageTable,
    PageDirectory,
    #[sel4_cfg(VTX)]
    VCpu,
    #[sel4_cfg(VTX)]
    EPTPageDirectory,
    #[sel4_cfg(VTX)]
    EPTPageTable,
    SeL4Arch(ObjectBlueprintSeL4Arch),
}

impl ObjectBlueprintX86 {
    pub(crate) const fn ty(self) -> ObjectTypeX86 {
        sel4_cfg_wrap_match! {
            match self {
                Self::_4k => ObjectTypeX86::_4k,
                Self::LargePage => ObjectTypeX86::LargePage,
                Self::PageTable => ObjectTypeX86::PageTable,
                Self::PageDirectory => ObjectTypeX86::PageDirectory,
                #[sel4_cfg(VTX)]
                Self::VCpu => ObjectTypeX86::VCpu,
                #[sel4_cfg(VTX)]
                Self::EPTPageDirectory => ObjectTypeX86::EPTPageDirectory,
                #[sel4_cfg(VTX)]
                Self::EPTPageTable => ObjectTypeX86::EPTPageTable,
                Self::SeL4Arch(sel4_arch) => ObjectTypeX86::SeL4Arch(sel4_arch.ty()),
            }
        }
    }

    pub(crate) const fn physical_size_bits(self) -> usize {
        sel4_cfg_wrap_match! {
            match self {
                Self::_4k => u32_into_usize(sys::seL4_PageBits),
                Self::LargePage => u32_into_usize(sys::seL4_LargePageBits),
                Self::PageTable => u32_into_usize(sys::seL4_PageTableBits),
                Self::PageDirectory => u32_into_usize(sys::seL4_PageDirBits),
                #[sel4_cfg(VTX)]
                Self::VCpu => u32_into_usize(sys::seL4_VCPUBits),
                #[sel4_cfg(VTX)]
                Self::EPTPageDirectory => u32_into_usize(sys::seL4_X86_EPTPDBits),
                #[sel4_cfg(VTX)]
                Self::EPTPageTable => u32_into_usize(sys::seL4_X86_EPTPTBits),
                Self::SeL4Arch(sel4_arch) => sel4_arch.physical_size_bits(),
            }
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
