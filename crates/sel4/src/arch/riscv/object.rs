//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use core::ffi::c_uint;

use sel4_config::{sel4_cfg_enum, sel4_cfg_wrap_match};

use crate::{const_helpers::u32_into_usize, sys};

pub type ObjectTypeArch = ObjectTypeRISCV;

pub type ObjectBlueprintArch = ObjectBlueprintRISCV;

#[sel4_cfg_enum]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ObjectTypeRISCV {
    _4KPage,
    MegaPage,
    PageTable,
    #[sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
    GigaPage,
}

impl ObjectTypeRISCV {
    pub(crate) const fn into_sys(self) -> c_uint {
        sel4_cfg_wrap_match! {
            match self {
                Self::_4KPage => sys::_object::seL4_RISCV_4K_Page,
                Self::MegaPage => sys::_object::seL4_RISCV_Mega_Page,
                #[sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
                Self::GigaPage => sys::_mode_object::seL4_RISCV_Giga_Page,
                Self::PageTable => sys::_object::seL4_RISCV_PageTableObject,
            }
        }
    }
}

#[sel4_cfg_enum]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectBlueprintRISCV {
    _4KPage,
    MegaPage,
    #[sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
    GigaPage,
    PageTable,
}

impl ObjectBlueprintRISCV {
    pub(crate) const fn ty(self) -> ObjectTypeRISCV {
        sel4_cfg_wrap_match! {
            match self {
                Self::_4KPage => ObjectTypeRISCV::_4KPage,
                Self::MegaPage => ObjectTypeRISCV::MegaPage,
                #[sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
                Self::GigaPage => ObjectTypeRISCV::GigaPage,
                Self::PageTable => ObjectTypeRISCV::PageTable,
            }
        }
    }

    pub(crate) const fn physical_size_bits(self) -> usize {
        sel4_cfg_wrap_match! {
            match self {
                Self::_4KPage => u32_into_usize(sys::seL4_PageBits),
                Self::MegaPage => u32_into_usize(sys::seL4_LargePageBits),
                #[sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
                Self::GigaPage => u32_into_usize(sys::seL4_HugePageBits),
                Self::PageTable => u32_into_usize(sys::seL4_PageTableBits),
            }
        }
    }
}
