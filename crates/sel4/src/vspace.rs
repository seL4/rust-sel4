use crate::{cap_type, CapType, FrameSize, Unspecified};

pub const GRANULE_SIZE: FrameSize = cap_type::Granule::FRAME_SIZE;

impl FrameSize {
    pub const fn bytes(self) -> usize {
        1 << self.bits()
    }
}

pub trait FrameType: CapType {
    const FRAME_SIZE: FrameSize;
}

#[derive(Copy, Clone, Debug)]
pub struct AnyFrame<C> {
    cptr: Unspecified<C>,
    size: FrameSize,
}

impl<C> AnyFrame<C> {
    pub fn cptr(self) -> Unspecified<C> {
        self.cptr
    }

    pub fn size(&self) -> &FrameSize {
        &self.size
    }
}
