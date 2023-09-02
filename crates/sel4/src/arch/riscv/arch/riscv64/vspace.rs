use crate::{cap_type, FrameType, ObjectBlueprint, ObjectBlueprintRISCV};

#[derive(Copy, Clone, Debug)]
pub enum FrameSize {
    _4K,
}

impl FrameSize {
    pub const fn blueprint(self) -> ObjectBlueprint {
        match self {
            FrameSize::_4K => ObjectBlueprint::Arch(ObjectBlueprintRISCV::_4KPage),
        }
    }
}

impl FrameType for cap_type::_4KPage {
    const FRAME_SIZE: FrameSize = FrameSize::_4K;
}
