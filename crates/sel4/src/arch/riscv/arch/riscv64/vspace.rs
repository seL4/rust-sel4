use crate::{cap_type, FrameType};

#[derive(Copy, Clone, Debug)]
pub enum FrameSize {
    _4KPage,
}

impl FrameSize {
    pub const fn bits(self) -> usize {
        match self {
            FrameSize::_4KPage => 12,
        }
    }
}

impl FrameType for cap_type::_4KPage {
    const FRAME_SIZE: FrameSize = FrameSize::_4KPage;
}
