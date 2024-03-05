//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use sel4_config::sel4_cfg_wrap_match;

#[allow(unused_imports)]
use crate::{cap_type, sys, FrameType, ObjectBlueprint, ObjectBlueprintRISCV, SizedFrameType};

#[sel4_config::sel4_cfg_enum]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FrameSize {
    _4K,
    Mega,
    #[sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
    Giga,
}

impl FrameSize {
    pub const fn blueprint(self) -> ObjectBlueprint {
        sel4_cfg_wrap_match! {
            match self {
                FrameSize::_4K => ObjectBlueprint::Arch(ObjectBlueprintRISCV::_4KPage),
                FrameSize::Mega => ObjectBlueprint::Arch(ObjectBlueprintRISCV::MegaPage),
                #[sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
                FrameSize::Giga => ObjectBlueprint::Arch(ObjectBlueprintRISCV::GigaPage),
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

impl FrameType for cap_type::_4KPage {}

impl SizedFrameType for cap_type::_4KPage {
    const FRAME_SIZE: FrameSize = FrameSize::_4K;
}

impl FrameType for cap_type::MegaPage {}

impl SizedFrameType for cap_type::MegaPage {
    const FRAME_SIZE: FrameSize = FrameSize::Mega;
}

#[sel4_config::sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
impl FrameType for cap_type::GigaPage {}

#[sel4_config::sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
impl SizedFrameType for cap_type::GigaPage {
    const FRAME_SIZE: FrameSize = FrameSize::Giga;
}

impl cap_type::PageTable {
    pub const INDEX_BITS: usize = sys::seL4_PageTableIndexBits as usize;
}
