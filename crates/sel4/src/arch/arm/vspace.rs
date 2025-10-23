//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use sel4_config::{sel4_cfg, sel4_cfg_enum, sel4_cfg_wrap_match};

use crate::{
    CapTypeForFrameObject, CapTypeForFrameObjectOfFixedSize, CapTypeForTranslationTableObject,
    ObjectBlueprint, ObjectBlueprintArm, cap_type, const_helpers::u32_into_usize, sys,
};

#[sel4_cfg(ARCH_AARCH64)]
use crate::ObjectBlueprintAArch64;

#[sel4_cfg(ARCH_AARCH32)]
use crate::ObjectBlueprintAArch32;

/// Frame object types for this kernel configuration.
#[sel4_cfg_enum]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FrameObjectType {
    SmallPage,
    LargePage,
    #[sel4_cfg(ARCH_AARCH64)]
    HugePage,
    #[sel4_cfg(ARCH_AARCH32)]
    Section,
}

impl FrameObjectType {
    pub const GRANULE: Self = Self::SmallPage;

    pub const fn blueprint(self) -> ObjectBlueprint {
        sel4_cfg_wrap_match! {
            match self {
                Self::SmallPage => ObjectBlueprint::Arch(ObjectBlueprintArm::SmallPage),
                Self::LargePage => ObjectBlueprint::Arch(ObjectBlueprintArm::LargePage),
                #[sel4_cfg(ARCH_AARCH64)]
                Self::HugePage => ObjectBlueprint::Arch(ObjectBlueprintArm::SeL4Arch(
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
                Self::SMALL_PAGE_BITS => Self::SmallPage,
                Self::LARGE_PAGE_BITS => Self::LargePage,
                #[sel4_cfg(ARCH_AARCH64)]
                Self::HUGE_PAGE_BITS => Self::HugePage,
                #[sel4_cfg(ARCH_AARCH32)]
                Self::SECTION_BITS => Self::Section,
                _ => return None,
            }
        })
    }

    // For match arm LHS's, as we can't call const fn's
    pub const SMALL_PAGE_BITS: usize = Self::SmallPage.bits();
    pub const LARGE_PAGE_BITS: usize = Self::LargePage.bits();

    #[sel4_cfg(ARCH_AARCH64)]
    pub const HUGE_PAGE_BITS: usize = Self::HugePage.bits();

    #[sel4_cfg(ARCH_AARCH32)]
    pub const SECTION_BITS: usize = Self::Section.bits();
}

impl CapTypeForFrameObject for cap_type::SmallPage {}

impl CapTypeForFrameObjectOfFixedSize for cap_type::SmallPage {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::SmallPage;
}

impl CapTypeForFrameObject for cap_type::LargePage {}

impl CapTypeForFrameObjectOfFixedSize for cap_type::LargePage {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::LargePage;
}

#[sel4_cfg(ARCH_AARCH64)]
impl CapTypeForFrameObject for cap_type::HugePage {}

#[sel4_cfg(ARCH_AARCH64)]
impl CapTypeForFrameObjectOfFixedSize for cap_type::HugePage {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::HugePage;
}

#[sel4_cfg(ARCH_AARCH32)]
impl CapTypeForFrameObject for cap_type::Section {}

#[sel4_cfg(ARCH_AARCH32)]
impl CapTypeForFrameObjectOfFixedSize for cap_type::Section {
    const FRAME_OBJECT_TYPE: FrameObjectType = FrameObjectType::Section;
}

// // //

/// Translation table object types for this kernel configuration.
#[sel4_cfg_enum]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TranslationTableObjectType {
    PT,
    #[sel4_cfg(ARCH_AARCH64)]
    VSpace,
    #[sel4_cfg(ARCH_AARCH32)]
    PD,
}

impl TranslationTableObjectType {
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

impl CapTypeForTranslationTableObject for cap_type::PT {
    const TRANSLATION_TABLE_OBJECT_TYPE: TranslationTableObjectType =
        TranslationTableObjectType::PT;
}

#[sel4_cfg(ARCH_AARCH64)]
impl CapTypeForTranslationTableObject for cap_type::VSpace {
    const TRANSLATION_TABLE_OBJECT_TYPE: TranslationTableObjectType =
        TranslationTableObjectType::VSpace;
}

#[sel4_cfg(ARCH_AARCH32)]
impl CapTypeForTranslationTableObject for cap_type::PD {
    const TRANSLATION_TABLE_OBJECT_TYPE: TranslationTableObjectType =
        TranslationTableObjectType::PD;
}

pub mod vspace_levels {
    use sel4_config::sel4_cfg_bool;

    pub const NUM_LEVELS: usize = if sel4_cfg_bool!(ARCH_AARCH64) { 4 } else { 2 };

    pub const HIGHEST_LEVEL_WITH_PAGE_ENTRIES: usize =
        NUM_LEVELS - if sel4_cfg_bool!(ARCH_AARCH64) { 3 } else { 2 };
}
