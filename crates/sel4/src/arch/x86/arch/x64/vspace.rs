//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use crate::{
    cap_type, sys, FrameType, ObjectBlueprint, ObjectBlueprintX64, ObjectBlueprintX86,
    SizedFrameType,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FrameSize {
    _4K,
    Large,
    Huge,
}

impl FrameSize {
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

impl FrameType for cap_type::_4K {}

impl SizedFrameType for cap_type::_4K {
    const FRAME_SIZE: FrameSize = FrameSize::_4K;
}

impl FrameType for cap_type::LargePage {}

impl SizedFrameType for cap_type::LargePage {
    const FRAME_SIZE: FrameSize = FrameSize::Large;
}

impl FrameType for cap_type::HugePage {}

impl SizedFrameType for cap_type::HugePage {
    const FRAME_SIZE: FrameSize = FrameSize::Huge;
}

//

impl cap_type::PML4 {
    pub const INDEX_BITS: usize = sys::seL4_PML4IndexBits as usize;
}

impl cap_type::PDPT {
    pub const INDEX_BITS: usize = sys::seL4_PDPTIndexBits as usize;
}

impl cap_type::PageDirectory {
    pub const INDEX_BITS: usize = sys::seL4_PageDirIndexBits as usize;
}

impl cap_type::PageTable {
    pub const INDEX_BITS: usize = sys::seL4_PageTableIndexBits as usize;
}
