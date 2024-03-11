//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use sel4_config::{sel4_cfg_usize, sel4_cfg_wrap_match};

#[allow(unused_imports)]
use crate::{
    cap_type, const_helpers::u32_into_usize, sys, CapTypeForFrameObject,
    CapTypeForFrameObjectOfFixedSize, CapTypeForTranslationStructureObject, ObjectBlueprint,
    ObjectBlueprintRISCV,
};

#[sel4_config::sel4_cfg_enum]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FrameObjectType {
    _4KPage,
    MegaPage,
    #[sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
    GigaPage,
}

impl FrameObjectType {
    pub const GRANULE: Self = Self::_4KPage;

    pub const fn blueprint(self) -> ObjectBlueprint {
        sel4_cfg_wrap_match! {
            match self {
                FrameObjectType::_4KPage => ObjectBlueprint::Arch(ObjectBlueprintRISCV::_4KPage),
                FrameObjectType::MegaPage => ObjectBlueprint::Arch(ObjectBlueprintRISCV::MegaPage),
                #[sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
                FrameObjectType::GigaPage => ObjectBlueprint::Arch(ObjectBlueprintRISCV::GigaPage),
            }
        }
    }

    pub const fn from_bits(bits: usize) -> Option<Self> {
        Some(sel4_cfg_wrap_match! {
            match bits {
                Self::_4K_PAGE_BITS => Self::_4KPage,
                Self::MEGA_PAGE_BITS => Self::MegaPage,
                #[sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
                Self::GIGA_PAGE_BITS => Self::GigaPage,
                _ => return None,
            }
        })
    }

    // For match arm LHS's, as we can't call const fn's

    pub const _4K_PAGE_BITS: usize = Self::_4KPage.bits();
    pub const MEGA_PAGE_BITS: usize = Self::MegaPage.bits();

    #[sel4_config::sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
    pub const GIGA_PAGE_BITS: usize = Self::GigaPage.bits();
}

impl CapTypeForFrameObject for cap_type::_4KPage {}

impl CapTypeForFrameObjectOfFixedSize for cap_type::_4KPage {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::_4KPage;
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
pub enum TranslationStructureObjectType {
    PageTable,
}

impl TranslationStructureObjectType {
    pub const NUM_LEVELS: usize = sel4_cfg_usize!(PT_LEVELS);

    pub const fn blueprint(&self) -> ObjectBlueprint {
        ObjectBlueprint::Arch(ObjectBlueprintRISCV::PageTable)
    }

    pub const fn index_bits(&self) -> usize {
        u32_into_usize(sys::seL4_PageTableIndexBits)
    }

    pub const fn from_level(level: usize) -> Option<Self> {
        if level < Self::NUM_LEVELS {
            Some(Self::PageTable)
        } else {
            None
        }
    }
}

impl CapTypeForTranslationStructureObject for cap_type::PageTable {
    const TRANSLATION_STRUCTURE_OBJECT_TYPE: TranslationStructureObjectType =
        TranslationStructureObjectType::PageTable;
}
