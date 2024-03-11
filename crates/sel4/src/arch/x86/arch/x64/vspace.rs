//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use crate::{
    cap_type, const_helpers::u32_into_usize, sys, CapTypeForFrameObject,
    CapTypeForFrameObjectOfFixedSize, CapTypeForTranslationStructureObject, ObjectBlueprint,
    ObjectBlueprintX64, ObjectBlueprintX86,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FrameObjectType {
    _4K,
    Large,
    Huge,
}

impl FrameObjectType {
    pub const GRANULE: Self = Self::_4K;

    pub const fn blueprint(self) -> ObjectBlueprint {
        match self {
            Self::_4K => ObjectBlueprint::Arch(ObjectBlueprintX86::_4K),
            Self::Large => ObjectBlueprint::Arch(ObjectBlueprintX86::LargePage),
            Self::Huge => {
                ObjectBlueprint::Arch(ObjectBlueprintX86::SeL4Arch(ObjectBlueprintX64::HugePage))
            }
        }
    }

    pub const fn from_bits(bits: usize) -> Option<Self> {
        Some(match bits {
            Self::_4K_BITS => Self::_4K,
            Self::LARGE_BITS => Self::Large,
            Self::HUGE_BITS => Self::Huge,
            _ => return None,
        })
    }

    // For match arm LHS's, as we can't call const fn's
    pub const _4K_BITS: usize = Self::_4K.bits();
    pub const LARGE_BITS: usize = Self::Large.bits();
    pub const HUGE_BITS: usize = Self::Huge.bits();
}

impl CapTypeForFrameObject for cap_type::_4K {}

impl CapTypeForFrameObjectOfFixedSize for cap_type::_4K {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::_4K;
}

impl CapTypeForFrameObject for cap_type::LargePage {}

impl CapTypeForFrameObjectOfFixedSize for cap_type::LargePage {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::Large;
}

impl CapTypeForFrameObject for cap_type::HugePage {}

impl CapTypeForFrameObjectOfFixedSize for cap_type::HugePage {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::Huge;
}

//

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TranslationStructureObjectType {
    PML4,
    PDPT,
    PageDirectory,
    PageTable,
}

impl TranslationStructureObjectType {
    pub const NUM_LEVELS: usize = 4;

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

impl CapTypeForTranslationStructureObject for cap_type::PML4 {
    const TRANSLATION_STRUCTURE_OBJECT_TYPE: TranslationStructureObjectType =
        TranslationStructureObjectType::PML4;
}

impl CapTypeForTranslationStructureObject for cap_type::PDPT {
    const TRANSLATION_STRUCTURE_OBJECT_TYPE: TranslationStructureObjectType =
        TranslationStructureObjectType::PDPT;
}

impl CapTypeForTranslationStructureObject for cap_type::PageDirectory {
    const TRANSLATION_STRUCTURE_OBJECT_TYPE: TranslationStructureObjectType =
        TranslationStructureObjectType::PageDirectory;
}

impl CapTypeForTranslationStructureObject for cap_type::PageTable {
    const TRANSLATION_STRUCTURE_OBJECT_TYPE: TranslationStructureObjectType =
        TranslationStructureObjectType::PageTable;
}
