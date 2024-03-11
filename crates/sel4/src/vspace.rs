//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2020 Arm Limited
//
// SPDX-License-Identifier: MIT
//

use crate::{
    cap_type, CapType, CapTypeForObjectOfFixedSize, FrameObjectType, TranslationStructureType,
};

impl FrameObjectType {
    pub const fn bits(self) -> usize {
        self.blueprint().physical_size_bits()
    }

    pub const fn bytes(self) -> usize {
        1 << self.bits()
    }
}

impl TranslationStructureType {
    pub fn span_bits(level: usize) -> usize {
        (level..Self::NUM_LEVELS)
            .map(|level| {
                TranslationStructureType::from_level(level)
                    .unwrap()
                    .index_bits()
            })
            .sum::<usize>()
            + FrameObjectType::GRANULE.bits()
    }
}

/// Trait for [`CapType`]s which correspond to frame objects.
pub trait CapTypeForFrameObject: CapType {}

impl CapTypeForFrameObject for cap_type::UnspecifiedFrame {}

/// Trait for [`CapTypeForFrameObject`]s which correspond to frame objects of fixed size.
pub trait CapTypeForFrameObjectOfFixedSize:
    CapTypeForObjectOfFixedSize + CapTypeForFrameObject
{
    const FRAME_OBJECT_TYPE: FrameObjectType;
}

pub trait CapTypeForTranslationStructureObject: CapTypeForObjectOfFixedSize {
    const TRANSLATION_STRUCTURE_TYPE: TranslationStructureType;
}
