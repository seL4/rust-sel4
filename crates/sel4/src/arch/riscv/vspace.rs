//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use sel4_config::sel4_cfg_wrap_match;

#[allow(unused_imports)]
use crate::{
    cap_type, const_helpers::u32_into_usize, sys, CapTypeForFrameObject,
    CapTypeForFrameObjectOfFixedSize, CapTypeForTranslationTableObject, ObjectBlueprint,
    ObjectBlueprintRiscV,
};

#[sel4_config::sel4_cfg_enum]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FrameObjectType {
    _4kPage,
    MegaPage,
    #[sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
    GigaPage,
}

impl FrameObjectType {
    pub const GRANULE: Self = Self::_4kPage;

    pub const fn blueprint(self) -> ObjectBlueprint {
        sel4_cfg_wrap_match! {
            match self {
                FrameObjectType::_4kPage => ObjectBlueprint::Arch(ObjectBlueprintRiscV::_4kPage),
                FrameObjectType::MegaPage => ObjectBlueprint::Arch(ObjectBlueprintRiscV::MegaPage),
                #[sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
                FrameObjectType::GigaPage => ObjectBlueprint::Arch(ObjectBlueprintRiscV::GigaPage),
            }
        }
    }

    pub const fn from_bits(bits: usize) -> Option<Self> {
        Some(sel4_cfg_wrap_match! {
            match bits {
                Self::_4K_PAGE_BITS => Self::_4kPage,
                Self::MEGA_PAGE_BITS => Self::MegaPage,
                #[sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
                Self::GIGA_PAGE_BITS => Self::GigaPage,
                _ => return None,
            }
        })
    }

    // For match arm LHS's, as we can't call const fn's

    pub const _4K_PAGE_BITS: usize = Self::_4kPage.bits();
    pub const MEGA_PAGE_BITS: usize = Self::MegaPage.bits();

    #[sel4_config::sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
    pub const GIGA_PAGE_BITS: usize = Self::GigaPage.bits();
}

impl CapTypeForFrameObject for cap_type::_4kPage {}

impl CapTypeForFrameObjectOfFixedSize for cap_type::_4kPage {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::_4kPage;
}

impl CapTypeForFrameObject for cap_type::MegaPage {}

impl CapTypeForFrameObjectOfFixedSize for cap_type::MegaPage {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::MegaPage;
}

#[sel4_config::sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
impl CapTypeForFrameObject for cap_type::GigaPage {}

#[sel4_config::sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
impl CapTypeForFrameObjectOfFixedSize for cap_type::GigaPage {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::GigaPage;
}

//

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TranslationTableObjectType {
    PageTable,
}

impl TranslationTableObjectType {
    pub const fn blueprint(&self) -> ObjectBlueprint {
        ObjectBlueprint::Arch(ObjectBlueprintRiscV::PageTable)
    }

    pub const fn index_bits(&self) -> usize {
        u32_into_usize(sys::seL4_PageTableIndexBits)
    }

    pub const fn from_level(level: usize) -> Option<Self> {
        if level < vspace_levels::NUM_LEVELS {
            Some(Self::PageTable)
        } else {
            None
        }
    }
}

impl CapTypeForTranslationTableObject for cap_type::PageTable {
    const TRANSLATION_TABLE_OBJECT_TYPE: TranslationTableObjectType =
        TranslationTableObjectType::PageTable;
}

pub mod vspace_levels {
    use sel4_config::sel4_cfg_usize;

    pub const NUM_LEVELS: usize = sel4_cfg_usize!(PT_LEVELS);

    pub const FIRST_LEVEL_WITH_FRAME_ENTRIES: usize = NUM_LEVELS
        - if sel4_cfg_usize!(PT_LEVELS) == 3 || sel4_cfg_usize!(PT_LEVELS) == 4 {
            3
        } else {
            2
        };
}
