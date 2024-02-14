//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2020 Arm Limited
//
// SPDX-License-Identifier: MIT
//

use crate::{cap_type, CapType, FrameSize};

/// The smallest [`FrameSize`].
pub const GRANULE_SIZE: FrameSize = cap_type::Granule::FRAME_SIZE;

impl FrameSize {
    pub const fn bits(self) -> usize {
        self.blueprint().physical_size_bits()
    }

    pub const fn bytes(self) -> usize {
        1 << self.bits()
    }
}

/// Trait for [`CapType`]s which correspond to frame objects.
pub trait FrameType: CapType {}

impl FrameType for cap_type::UnspecifiedFrame {}

/// Trait for [`FrameType`]s which correspond to frame objects of fixed size.
pub trait SizedFrameType: FrameType {
    const FRAME_SIZE: FrameSize;
}
