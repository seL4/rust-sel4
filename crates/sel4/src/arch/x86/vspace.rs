//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use sel4_config::{sel4_cfg, sel4_cfg_enum, sel4_cfg_wrap_match};

use crate::{
    CapTypeForFrameObject, CapTypeForFrameObjectOfFixedSize, CapTypeForTranslationTableObject,
    ObjectBlueprint, ObjectBlueprintX64, ObjectBlueprintX86, cap_type,
    const_helpers::u32_into_usize, sys,
};

/// Frame object types for this kernel configuration.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FrameObjectType {
    _4k,
    LargePage,
    HugePage,
}

impl FrameObjectType {
    pub const GRANULE: Self = Self::_4k;

    pub const fn blueprint(self) -> ObjectBlueprint {
        match self {
            Self::_4k => ObjectBlueprint::Arch(ObjectBlueprintX86::_4k),
            Self::LargePage => ObjectBlueprint::Arch(ObjectBlueprintX86::LargePage),
            Self::HugePage => {
                ObjectBlueprint::Arch(ObjectBlueprintX86::SeL4Arch(ObjectBlueprintX64::HugePage))
            }
        }
    }

    pub const fn from_bits(bits: usize) -> Option<Self> {
        Some(match bits {
            Self::_4K_BITS => Self::_4k,
            Self::LARGE_PAGE_BITS => Self::LargePage,
            Self::HUGE_PAGE_BITS => Self::HugePage,
            _ => return None,
        })
    }

    // For match arm LHS's, as we can't call const fn's
    pub const _4K_BITS: usize = Self::_4k.bits();
    pub const LARGE_PAGE_BITS: usize = Self::LargePage.bits();
    pub const HUGE_PAGE_BITS: usize = Self::HugePage.bits();
}

impl CapTypeForFrameObject for cap_type::_4k {}

impl CapTypeForFrameObjectOfFixedSize for cap_type::_4k {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::_4k;
}

impl CapTypeForFrameObject for cap_type::LargePage {}

impl CapTypeForFrameObjectOfFixedSize for cap_type::LargePage {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::LargePage;
}

impl CapTypeForFrameObject for cap_type::HugePage {}

impl CapTypeForFrameObjectOfFixedSize for cap_type::HugePage {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::HugePage;
}

// // //

/// Translation table object types for this kernel configuration.
#[sel4_cfg_enum]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TranslationTableObjectType {
    PML4,
    PDPT,
    PageDirectory,
    PageTable,
    #[sel4_cfg(VTX)]
    EPTPML4,
    #[sel4_cfg(VTX)]
    EPTPDPT,
    #[sel4_cfg(VTX)]
    EPTPageDirectory,
    #[sel4_cfg(VTX)]
    EPTPageTable,
}

impl TranslationTableObjectType {
    pub const fn blueprint(&self) -> ObjectBlueprint {
        sel4_cfg_wrap_match! {
            match self {
                Self::PML4 => {
                    ObjectBlueprint::Arch(ObjectBlueprintX86::SeL4Arch(ObjectBlueprintX64::PML4))
                }
                Self::PDPT => {
                    ObjectBlueprint::Arch(ObjectBlueprintX86::SeL4Arch(ObjectBlueprintX64::PDPT))
                }
                Self::PageDirectory => ObjectBlueprint::Arch(ObjectBlueprintX86::PageDirectory),
                Self::PageTable => ObjectBlueprint::Arch(ObjectBlueprintX86::PageTable),
                #[sel4_cfg(VTX)]
                Self::EPTPML4 => {
                    ObjectBlueprint::Arch(ObjectBlueprintX86::SeL4Arch(ObjectBlueprintX64::EPTPML4))
                }
                #[sel4_cfg(VTX)]
                Self::EPTPDPT => {
                    ObjectBlueprint::Arch(ObjectBlueprintX86::SeL4Arch(ObjectBlueprintX64::EPTPDPT))
                }
                #[sel4_cfg(VTX)]
                Self::EPTPageDirectory => ObjectBlueprint::Arch(ObjectBlueprintX86::EPTPageDirectory),
                #[sel4_cfg(VTX)]
                Self::EPTPageTable => ObjectBlueprint::Arch(ObjectBlueprintX86::EPTPageTable),
            }
        }
    }

    pub const fn index_bits(&self) -> usize {
        sel4_cfg_wrap_match! {
            match self {
                Self::PML4 => u32_into_usize(sys::seL4_PML4IndexBits),
                Self::PDPT => u32_into_usize(sys::seL4_PDPTIndexBits),
                Self::PageDirectory => u32_into_usize(sys::seL4_PageDirIndexBits),
                Self::PageTable => u32_into_usize(sys::seL4_PageTableIndexBits),
                #[sel4_cfg(VTX)]
                Self::EPTPML4 => u32_into_usize(sys::seL4_X86_EPTPML4IndexBits),
                #[sel4_cfg(VTX)]
                Self::EPTPDPT => u32_into_usize(sys::seL4_X86_EPTPDPTIndexBits),
                #[sel4_cfg(VTX)]
                Self::EPTPageDirectory => u32_into_usize(sys::seL4_X86_EPTPDIndexBits),
                #[sel4_cfg(VTX)]
                Self::EPTPageTable => u32_into_usize(sys::seL4_X86_EPTPTIndexBits),
            }
        }
    }

    pub const fn from_level(level: usize) -> Option<Self> {
        Some(match level {
            0 => Self::PML4,
            1 => Self::PDPT,
            2 => Self::PageDirectory,
            3 => Self::PageTable,
            _ => return None,
        })
    }

    #[sel4_cfg(VTX)]
    pub const fn from_level_ept(level: usize) -> Option<Self> {
        Some(match level {
            0 => Self::EPTPML4,
            1 => Self::EPTPDPT,
            2 => Self::EPTPageDirectory,
            3 => Self::EPTPageTable,
            _ => return None,
        })
    }
}

impl CapTypeForTranslationTableObject for cap_type::PML4 {
    const TRANSLATION_TABLE_OBJECT_TYPE: TranslationTableObjectType =
        TranslationTableObjectType::PML4;
}

impl CapTypeForTranslationTableObject for cap_type::PDPT {
    const TRANSLATION_TABLE_OBJECT_TYPE: TranslationTableObjectType =
        TranslationTableObjectType::PDPT;
}

impl CapTypeForTranslationTableObject for cap_type::PageDirectory {
    const TRANSLATION_TABLE_OBJECT_TYPE: TranslationTableObjectType =
        TranslationTableObjectType::PageDirectory;
}

impl CapTypeForTranslationTableObject for cap_type::PageTable {
    const TRANSLATION_TABLE_OBJECT_TYPE: TranslationTableObjectType =
        TranslationTableObjectType::PageTable;
}

pub mod vspace_levels {
    pub const NUM_LEVELS: usize = 4;

    pub const HIGHEST_LEVEL_WITH_PAGE_ENTRIES: usize = NUM_LEVELS - 3;
}
