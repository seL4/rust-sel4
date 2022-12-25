use crate::{cap_type, FrameType, ObjectBlueprint};

#[derive(Copy, Clone, Debug)]
pub enum FrameSize {
    _4KPage,
}

impl FrameSize {
    pub const fn blueprint(self) -> ObjectBlueprint {
        match self {
            FrameSize::_4KPage => todo!(),
        }
    }
}

impl FrameType for cap_type::_4KPage {
    const FRAME_SIZE: FrameSize = FrameSize::_4KPage;
}
