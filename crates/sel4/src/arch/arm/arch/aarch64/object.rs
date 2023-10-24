//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use core::ffi::c_uint;

use crate::{const_helpers::u32_into_usize, sys};

/// Alias for [`ObjectTypeAArch64`].
pub type ObjectTypeSeL4Arch = ObjectTypeAArch64;

/// Alias for [`ObjectBlueprintAArch64`].
pub type ObjectBlueprintSeL4Arch = ObjectBlueprintAArch64;

/// Corresponds to `seL4_ModeObjectType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ObjectTypeAArch64 {
    HugePage,
    VSpace,
}

impl ObjectTypeAArch64 {
    pub(crate) const fn into_sys(self) -> c_uint {
        match self {
            Self::HugePage => sys::_mode_object::seL4_ARM_HugePageObject,
            Self::VSpace => sys::_mode_object::seL4_ARM_VSpaceObject,
        }
    }
}

/// AArch64-specific variants of [`ObjectBlueprint`](crate::ObjectBlueprint).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectBlueprintAArch64 {
    HugePage,
    VSpace,
}

impl ObjectBlueprintAArch64 {
    pub(crate) const fn ty(self) -> ObjectTypeAArch64 {
        match self {
            Self::HugePage => ObjectTypeAArch64::HugePage,
            Self::VSpace => ObjectTypeAArch64::VSpace,
        }
    }

    pub(crate) const fn physical_size_bits(self) -> usize {
        match self {
            Self::HugePage => u32_into_usize(sys::seL4_HugePageBits),
            Self::VSpace => u32_into_usize(sys::seL4_VSpaceBits),
        }
    }
}
