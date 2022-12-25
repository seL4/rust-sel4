use crate::{cap_type, FrameType, ObjectBlueprint};

#[derive(Copy, Clone, Debug)]
pub enum FrameSize {
    _4K,
}

impl FrameSize {
    pub const fn blueprint(self) -> ObjectBlueprint {
        match self {
            FrameSize::_4K => todo!(),
        }
    }
}

impl FrameType for cap_type::_4K {
    const FRAME_SIZE: FrameSize = FrameSize::_4K;
}
