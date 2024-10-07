//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2020 Arm Limited
//
// SPDX-License-Identifier: MIT
//

use crate::{
    cap_type, CapType, CapTypeForObjectOfFixedSize, FrameObjectType, TranslationTableObjectType,
};

impl FrameObjectType {
    pub const fn bits(self) -> usize {
        self.blueprint().physical_size_bits()
    }

    pub const fn bytes(self) -> usize {
        1 << self.bits()
    }
}

/// Trait for [`CapType`]s which correspond to frame objects.
pub trait CapTypeForFrameObject: CapType {}

impl CapTypeForFrameObject for cap_type::UnspecifiedPage {}

/// Trait for [`CapType`]s which correspond to frame objects of fixed size.
pub trait CapTypeForFrameObjectOfFixedSize:
    CapTypeForObjectOfFixedSize + CapTypeForFrameObject
{
    const FRAME_OBJECT_TYPE: FrameObjectType;
}

/// Trait for [`CapType`]s which correspond to translation table objects.
pub trait CapTypeForTranslationTableObject: CapTypeForObjectOfFixedSize {
    const TRANSLATION_TABLE_OBJECT_TYPE: TranslationTableObjectType;
}

/// Items describing the layout of address translation structures for this kernel configuration.
pub mod vspace_levels {
    use crate::{FrameObjectType, TranslationTableObjectType};

    /// The maximum number of levels of translation tables for this kernel configuration.
    pub use crate::arch::vspace_levels::NUM_LEVELS;

    /// Lowest level of translation table whose entries can be pages rather than lower-level translation tables.
    pub use crate::arch::vspace_levels::FIRST_LEVEL_WITH_FRAME_ENTRIES;

    /// The number of address bits spanned by the given translation table level.
    pub fn span_bits(level: usize) -> usize {
        assert!(level < NUM_LEVELS);
        (level..NUM_LEVELS)
            .map(|level| {
                TranslationTableObjectType::from_level(level)
                    .unwrap()
                    .index_bits()
            })
            .sum::<usize>()
            + FrameObjectType::GRANULE.bits()
    }

    /// The number of address bits spanned by entries of the given translation table level.
    pub fn step_bits(level: usize) -> usize {
        span_bits(level)
            - TranslationTableObjectType::from_level(level)
                .unwrap()
                .index_bits()
    }
}
