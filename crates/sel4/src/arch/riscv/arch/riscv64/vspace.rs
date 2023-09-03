use crate::{
    cap_type, sys, FrameType, ObjectBlueprint, ObjectBlueprintRISCV, ObjectBlueprintRISCV64,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FrameSize {
    _4K,
    Mega,
    Giga,
}

impl FrameSize {
    pub const fn blueprint(self) -> ObjectBlueprint {
        match self {
            FrameSize::_4K => ObjectBlueprint::Arch(ObjectBlueprintRISCV::_4KPage),
            FrameSize::Mega => ObjectBlueprint::Arch(ObjectBlueprintRISCV::MegaPage),
            FrameSize::Giga => ObjectBlueprint::Arch(ObjectBlueprintRISCV::SeL4Arch(
                ObjectBlueprintRISCV64::GigaPage,
            )),
        }
    }

    // For match arm LHS's, as we can't call const fn's
    pub const _4K_BITS: usize = Self::_4K.bits();
    pub const MEGA_BITS: usize = Self::Mega.bits();
    pub const GIGA_BITS: usize = Self::Giga.bits();
}

impl FrameType for cap_type::_4KPage {
    const FRAME_SIZE: FrameSize = FrameSize::_4K;
}

impl FrameType for cap_type::MegaPage {
    const FRAME_SIZE: FrameSize = FrameSize::Mega;
}

impl FrameType for cap_type::GigaPage {
    const FRAME_SIZE: FrameSize = FrameSize::Giga;
}

impl cap_type::PageTable {
    pub const INDEX_BITS: usize = sys::seL4_PageTableIndexBits as usize;
}
