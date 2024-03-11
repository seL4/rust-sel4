//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use sel4_config::{sel4_cfg, sel4_cfg_bool, sel4_cfg_enum, sel4_cfg_wrap_match};

use crate::{
    cap_type, const_helpers::u32_into_usize, sys, CapTypeForFrameObject,
    CapTypeForFrameObjectOfFixedSize, CapTypeForTranslationStructureObject, ObjectBlueprint,
    ObjectBlueprintArm,
};

#[sel4_cfg(ARCH_AARCH64)]
use crate::ObjectBlueprintAArch64;

#[sel4_cfg(ARCH_AARCH32)]
use crate::ObjectBlueprintAArch32;

/// Frame sizes for AArch64.
#[sel4_cfg_enum]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FrameObjectType {
    Small,
    Large,
    #[sel4_cfg(ARCH_AARCH64)]
    Huge,
    #[sel4_cfg(ARCH_AARCH32)]
    Section,
}

impl FrameObjectType {
    pub const GRANULE: Self = Self::Small;

    pub const fn blueprint(self) -> ObjectBlueprint {
        sel4_cfg_wrap_match! {
            match self {
                Self::Small => ObjectBlueprint::Arch(ObjectBlueprintArm::SmallPage),
                Self::Large => ObjectBlueprint::Arch(ObjectBlueprintArm::LargePage),
                #[sel4_cfg(ARCH_AARCH64)]
                Self::Huge => ObjectBlueprint::Arch(ObjectBlueprintArm::SeL4Arch(
                    ObjectBlueprintAArch64::HugePage,
                )),
                #[sel4_cfg(ARCH_AARCH32)]
                Self::Section => ObjectBlueprint::Arch(ObjectBlueprintArm::SeL4Arch(
                    ObjectBlueprintAArch32::Section,
                )),
            }
        }
    }

    pub const fn from_bits(bits: usize) -> Option<Self> {
        Some(sel4_cfg_wrap_match! {
            match bits {
                Self::SMALL_BITS => Self::Small,
                Self::LARGE_BITS => Self::Large,
                #[sel4_cfg(ARCH_AARCH64)]
                Self::HUGE_BITS => Self::Huge,
                #[sel4_cfg(ARCH_AARCH32)]
                Self::SECTION_BITS => Self::Section,
                _ => return None,
            }
        })
    }

    // For match arm LHS's, as we can't call const fn's
    pub const SMALL_BITS: usize = Self::Small.bits();
    pub const LARGE_BITS: usize = Self::Large.bits();

    #[sel4_cfg(ARCH_AARCH64)]
    pub const HUGE_BITS: usize = Self::Huge.bits();

    #[sel4_cfg(ARCH_AARCH32)]
    pub const SECTION_BITS: usize = Self::Section.bits();
}

impl CapTypeForFrameObject for cap_type::SmallPage {}

impl CapTypeForFrameObjectOfFixedSize for cap_type::SmallPage {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::Small;
}

impl CapTypeForFrameObject for cap_type::LargePage {}

impl CapTypeForFrameObjectOfFixedSize for cap_type::LargePage {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::Large;
}

#[sel4_cfg(ARCH_AARCH64)]
impl CapTypeForFrameObject for cap_type::HugePage {}

#[sel4_cfg(ARCH_AARCH64)]
impl CapTypeForFrameObjectOfFixedSize for cap_type::HugePage {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::Huge;
}

#[sel4_cfg(ARCH_AARCH32)]
impl CapTypeForFrameObject for cap_type::Section {}

#[sel4_cfg(ARCH_AARCH32)]
impl CapTypeForFrameObjectOfFixedSize for cap_type::Section {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::Section;
}

//

#[sel4_cfg_enum]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TranslationStructureObjectType {
    PT,
    #[sel4_cfg(ARCH_AARCH64)]
    VSpace,
    #[sel4_cfg(ARCH_AARCH32)]
    PD,
}

impl TranslationStructureObjectType {
    pub const NUM_LEVELS: usize = if sel4_cfg_bool!(ARCH_AARCH64) { 4 } else { 2 };

    pub const fn blueprint(&self) -> ObjectBlueprint {
        sel4_cfg_wrap_match! {
            match self {
                Self::PT => ObjectBlueprint::Arch(ObjectBlueprintArm::PT),
                #[sel4_cfg(ARCH_AARCH64)]
                Self::VSpace => ObjectBlueprint::Arch(ObjectBlueprintArm::SeL4Arch(
                    ObjectBlueprintAArch64::VSpace,
                )),
                #[sel4_cfg(ARCH_AARCH32)]
                Self::PD => ObjectBlueprint::Arch(ObjectBlueprintArm::SeL4Arch(
                    ObjectBlueprintAArch32::PD,
                )),
            }
        }
    }

    pub const fn index_bits(&self) -> usize {
        sel4_cfg_wrap_match! {
            match self {
                Self::PT => u32_into_usize(sys::seL4_PageTableIndexBits),
                #[sel4_cfg(ARCH_AARCH64)]
                Self::VSpace => u32_into_usize(sys::seL4_VSpaceIndexBits),
                #[sel4_cfg(ARCH_AARCH32)]
                Self::PD => u32_into_usize(sys::seL4_PageDirIndexBits),
            }
        }
    }

    pub const fn from_level(level: usize) -> Option<Self> {
        Some(sel4_cfg_wrap_match! {
            match level {
                #[sel4_cfg(ARCH_AARCH64)]
                0 => Self::VSpace,
                #[sel4_cfg(ARCH_AARCH32)]
                0 => Self::PD,
                #[sel4_cfg(ARCH_AARCH64)]
                1..=3 => Self::PT,
                #[sel4_cfg(ARCH_AARCH32)]
                1 => Self::PT,
                _ => return None,
            }
        })
    }
}

impl CapTypeForTranslationStructureObject for cap_type::PT {
    const TRANSLATION_STRUCTURE_OBJECT_TYPE: TranslationStructureObjectType =
        TranslationStructureObjectType::PT;
}

#[sel4_cfg(ARCH_AARCH64)]
impl CapTypeForTranslationStructureObject for cap_type::VSpace {
    const TRANSLATION_STRUCTURE_OBJECT_TYPE: TranslationStructureObjectType =
        TranslationStructureObjectType::VSpace;
}

#[sel4_cfg(ARCH_AARCH32)]
impl CapTypeForTranslationStructureObject for cap_type::PD {
    const TRANSLATION_STRUCTURE_OBJECT_TYPE: TranslationStructureObjectType =
        TranslationStructureObjectType::PD;
}
