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

pub trait FrameType: CapType {
    const FRAME_SIZE: FrameSize;
}
