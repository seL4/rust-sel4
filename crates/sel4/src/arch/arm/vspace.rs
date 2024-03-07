//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use sel4_config::{sel4_cfg, sel4_cfg_enum, sel4_cfg_wrap_match};

use crate::{
    cap_type, sys, CapTypeForFrameObject, CapTypeForFrameObjectOfFixedSize, ObjectBlueprint,
    ObjectBlueprintArm,
};

#[sel4_cfg(ARCH_AARCH64)]
use crate::ObjectBlueprintAArch64;

#[sel4_cfg(ARCH_AARCH32)]
use crate::ObjectBlueprintAArch32;

/// Frame sizes for AArch64.
#[sel4_cfg_enum]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FrameSize {
    Small,
    Large,
    #[sel4_cfg(ARCH_AARCH64)]
    Huge,
    #[sel4_cfg(ARCH_AARCH32)]
    Section,
}

impl FrameSize {
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
    const FRAME_SIZE: FrameSize = FrameSize::Small;
}

impl CapTypeForFrameObject for cap_type::LargePage {}

impl CapTypeForFrameObjectOfFixedSize for cap_type::LargePage {
    const FRAME_SIZE: FrameSize = FrameSize::Large;
}

#[sel4_cfg(ARCH_AARCH64)]
impl CapTypeForFrameObject for cap_type::HugePage {}

#[sel4_cfg(ARCH_AARCH64)]
impl CapTypeForFrameObjectOfFixedSize for cap_type::HugePage {
    const FRAME_SIZE: FrameSize = FrameSize::Huge;
}

#[sel4_cfg(ARCH_AARCH32)]
impl CapTypeForFrameObject for cap_type::Section {}

#[sel4_cfg(ARCH_AARCH32)]
impl CapTypeForFrameObjectOfFixedSize for cap_type::Section {
    const FRAME_SIZE: FrameSize = FrameSize::Section;
}

//

#[sel4_cfg(ARCH_AARCH64)]
impl cap_type::VSpace {
    pub const INDEX_BITS: usize = sys::seL4_VSpaceIndexBits as usize;
}

#[sel4_cfg(ARCH_AARCH32)]
impl cap_type::PD {
    pub const INDEX_BITS: usize = sys::seL4_PageDirIndexBits as usize;
}

impl cap_type::PT {
    pub const INDEX_BITS: usize = sys::seL4_PageTableIndexBits as usize;
}
