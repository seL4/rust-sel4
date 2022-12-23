use crate::{cap_type, FrameType};

#[derive(Copy, Clone, Debug)]
pub enum FrameSize {
    _4K,
}

impl FrameSize {
    pub const fn bits(self) -> usize {
        match self {
            FrameSize::_4K => 12,
        }
    }
}

impl FrameType for cap_type::_4K {
    const FRAME_SIZE: FrameSize = FrameSize::_4K;
}
