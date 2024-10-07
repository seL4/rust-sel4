//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use crate::{
    cap_type, const_helpers::u32_into_usize, sys, CapTypeForFrameObject,
    CapTypeForFrameObjectOfFixedSize, CapTypeForTranslationTableObject, ObjectBlueprint,
    ObjectBlueprintX64, ObjectBlueprintX86,
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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TranslationTableObjectType {
    PML4,
    PDPT,
    PageDirectory,
    PageTable,
}

impl TranslationTableObjectType {
    pub const fn blueprint(&self) -> ObjectBlueprint {
        match self {
            Self::PML4 => {
                ObjectBlueprint::Arch(ObjectBlueprintX86::SeL4Arch(ObjectBlueprintX64::PML4))
            }
            Self::PDPT => {
                ObjectBlueprint::Arch(ObjectBlueprintX86::SeL4Arch(ObjectBlueprintX64::PDPT))
            }
            Self::PageDirectory => ObjectBlueprint::Arch(ObjectBlueprintX86::PageDirectory),
            Self::PageTable => ObjectBlueprint::Arch(ObjectBlueprintX86::PageTable),
        }
    }

    pub const fn index_bits(&self) -> usize {
        match self {
            Self::PML4 => u32_into_usize(sys::seL4_PML4IndexBits),
            Self::PDPT => u32_into_usize(sys::seL4_PDPTIndexBits),
            Self::PageDirectory => u32_into_usize(sys::seL4_PageDirIndexBits),
            Self::PageTable => u32_into_usize(sys::seL4_PageTableIndexBits),
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
