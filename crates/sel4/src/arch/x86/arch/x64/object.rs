//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use core::ffi::c_uint;

use sel4_config::{sel4_cfg_enum, sel4_cfg_wrap_match};

use crate::{const_helpers::u32_into_usize, sys};

pub type ObjectTypeSeL4Arch = ObjectTypeX64;

pub type ObjectBlueprintSeL4Arch = ObjectBlueprintX64;

#[sel4_cfg_enum]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ObjectTypeX64 {
    HugePage,
    PDPT,
    PML4,
    #[sel4_cfg(VTX)]
    EPTPDPT,
    #[sel4_cfg(VTX)]
    EPTPML4,
}

impl ObjectTypeX64 {
    pub(crate) const fn into_sys(self) -> c_uint {
        sel4_cfg_wrap_match! {
            match self {
                Self::HugePage => sys::_mode_object::seL4_X64_HugePageObject,
                Self::PDPT => sys::_mode_object::seL4_X86_PDPTObject,
                Self::PML4 => sys::_mode_object::seL4_X64_PML4Object,
                #[sel4_cfg(VTX)]
                Self::EPTPDPT => sys::_object::seL4_X86_EPTPDPTObject,
                #[sel4_cfg(VTX)]
                Self::EPTPML4 => sys::_object::seL4_X86_EPTPML4Object,
            }
        }
    }
}

#[sel4_cfg_enum]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectBlueprintX64 {
    HugePage,
    PDPT,
    PML4,
    #[sel4_cfg(VTX)]
    EPTPDPT,
    #[sel4_cfg(VTX)]
    EPTPML4,
}

impl ObjectBlueprintX64 {
    pub(crate) const fn ty(self) -> ObjectTypeX64 {
        sel4_cfg_wrap_match! {
            match self {
                Self::HugePage => ObjectTypeX64::HugePage,
                Self::PDPT => ObjectTypeX64::PDPT,
                Self::PML4 => ObjectTypeX64::PML4,
                #[sel4_cfg(VTX)]
                Self::EPTPDPT => ObjectTypeX64::EPTPDPT,
                #[sel4_cfg(VTX)]
                Self::EPTPML4 => ObjectTypeX64::EPTPML4,
            }
        }
    }

    pub(crate) const fn physical_size_bits(self) -> usize {
        sel4_cfg_wrap_match! {
            match self {
                Self::HugePage => u32_into_usize(sys::seL4_HugePageBits),
                Self::PDPT => u32_into_usize(sys::seL4_PDPTBits),
                Self::PML4 => u32_into_usize(sys::seL4_PML4Bits),
                #[sel4_cfg(VTX)]
                Self::EPTPDPT => u32_into_usize(sys::seL4_X86_EPTPDPTBits),
                #[sel4_cfg(VTX)]
                Self::EPTPML4 => u32_into_usize(sys::seL4_X86_EPTPML4Bits),
            }
        }
    }
}
