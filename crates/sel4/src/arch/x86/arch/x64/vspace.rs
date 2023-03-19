use crate::{cap_type, sys, FrameType, ObjectBlueprint, ObjectBlueprintX64, ObjectBlueprintX86};

#[derive(Copy, Clone, Debug)]
pub enum FrameSize {
    _4K,
    Large,
    Huge,
}

impl FrameSize {
    pub const fn blueprint(self) -> ObjectBlueprint {
        match self {
            Self::_4K => ObjectBlueprintX86::_4K.into(),
            Self::Large => ObjectBlueprintX86::LargePage.into(),
            Self::Huge => ObjectBlueprintX64::HugePage.into(),
        }
    }
}

impl FrameType for cap_type::_4K {
    const FRAME_SIZE: FrameSize = FrameSize::_4K;
}

impl FrameType for cap_type::LargePage {
    const FRAME_SIZE: FrameSize = FrameSize::Large;
}

impl FrameType for cap_type::HugePage {
    const FRAME_SIZE: FrameSize = FrameSize::Huge;
}

//

impl cap_type::PDPT {
    pub const SPAN_BITS: usize =
        cap_type::PageDirectory::SPAN_BITS + (sys::seL4_PDPTIndexBits as usize);
}

impl cap_type::PageDirectory {
    pub const SPAN_BITS: usize =
        cap_type::PageTable::SPAN_BITS + (sys::seL4_PageDirIndexBits as usize);
}

impl cap_type::PageTable {
    pub const SPAN_BITS: usize = FrameSize::_4K.bits() + (sys::seL4_PageTableIndexBits as usize);
}
