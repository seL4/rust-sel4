//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use core::ffi::c_uint;

use crate::{const_helpers::u32_into_usize, sys};

/// Alias for [`ObjectTypeAArch32`].
pub type ObjectTypeSeL4Arch = ObjectTypeAArch32;

/// Alias for [`ObjectBlueprintAArch32`].
pub type ObjectBlueprintSeL4Arch = ObjectBlueprintAArch32;

/// Corresponds to `seL4_ModeObjectType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ObjectTypeAArch32 {
    Section,
    PD,
}

impl ObjectTypeAArch32 {
    pub(crate) const fn into_sys(self) -> c_uint {
        match self {
            Self::Section => sys::_object::seL4_ARM_SectionObject,
            Self::PD => sys::_mode_object::seL4_ARM_PageDirectoryObject,
        }
    }
}

/// AArch32-specific variants of [`ObjectBlueprint`](crate::ObjectBlueprint).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectBlueprintAArch32 {
    Section,
    PD,
}

impl ObjectBlueprintAArch32 {
    pub(crate) const fn ty(self) -> ObjectTypeAArch32 {
        match self {
            Self::Section => ObjectTypeAArch32::Section,
            Self::PD => ObjectTypeAArch32::PD,
        }
    }

    pub(crate) const fn physical_size_bits(self) -> usize {
        match self {
            Self::Section => u32_into_usize(sys::seL4_SectionBits),
            Self::PD => u32_into_usize(sys::seL4_PageDirBits),
        }
    }
}
