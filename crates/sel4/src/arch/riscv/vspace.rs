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
    _4K,
    Mega,
    #[sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
    Giga,
}

impl FrameObjectType {
    pub const GRANULE: Self = Self::_4K;

    pub const fn blueprint(self) -> ObjectBlueprint {
        sel4_cfg_wrap_match! {
            match self {
                FrameObjectType::_4K => ObjectBlueprint::Arch(ObjectBlueprintRISCV::_4KPage),
                FrameObjectType::Mega => ObjectBlueprint::Arch(ObjectBlueprintRISCV::MegaPage),
                #[sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
                FrameObjectType::Giga => ObjectBlueprint::Arch(ObjectBlueprintRISCV::GigaPage),
            }
        }
    }

    pub const fn from_bits(bits: usize) -> Option<Self> {
        Some(sel4_cfg_wrap_match! {
            match bits {
                Self::_4K_BITS => Self::_4K,
                Self::MEGA_BITS => Self::Mega,
                #[sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
                Self::GIGA_BITS => Self::Giga,
                _ => return None,
            }
        })
    }

    // For match arm LHS's, as we can't call const fn's

    pub const _4K_BITS: usize = Self::_4K.bits();
    pub const MEGA_BITS: usize = Self::Mega.bits();

    #[sel4_config::sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
    pub const GIGA_BITS: usize = Self::Giga.bits();
}

impl CapTypeForFrameObject for cap_type::_4KPage {}

impl CapTypeForFrameObjectOfFixedSize for cap_type::_4KPage {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::_4K;
}

impl CapTypeForFrameObject for cap_type::MegaPage {}

impl CapTypeForFrameObjectOfFixedSize for cap_type::MegaPage {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::Mega;
}

#[sel4_config::sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
impl CapTypeForFrameObject for cap_type::GigaPage {}

#[sel4_config::sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
impl CapTypeForFrameObjectOfFixedSize for cap_type::GigaPage {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::Giga;
}

//

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TranslationStructureType {
    PageTable,
}

impl TranslationStructureType {
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
    const TRANSLATION_STRUCTURE_TYPE: TranslationStructureType =
        TranslationStructureType::PageTable;
}
