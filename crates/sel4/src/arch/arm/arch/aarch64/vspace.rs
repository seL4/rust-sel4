use crate::{
    cap_type, sys, FrameType, ObjectBlueprint, ObjectBlueprintAArch64, ObjectBlueprintArm,
};

/// Frame sizes for AArch64.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FrameSize {
    Small,
    Large,
    Huge,
}

impl FrameSize {
    pub const fn blueprint(self) -> ObjectBlueprint {
        match self {
            Self::Small => ObjectBlueprintArm::SmallPage.into(),
            Self::Large => ObjectBlueprintArm::LargePage.into(),
            Self::Huge => ObjectBlueprintAArch64::HugePage.into(),
        }
    }

    // For match arm LHS's, as we can't call const fn's
    pub const SMALL_BITS: usize = Self::Small.bits();
    pub const LARGE_BITS: usize = Self::Large.bits();
    pub const HUGE_BITS: usize = Self::Huge.bits();
}

impl FrameType for cap_type::SmallPage {
    const FRAME_SIZE: FrameSize = FrameSize::Small;
}

impl FrameType for cap_type::LargePage {
    const FRAME_SIZE: FrameSize = FrameSize::Large;
}

impl FrameType for cap_type::HugePage {
    const FRAME_SIZE: FrameSize = FrameSize::Huge;
}

//

impl cap_type::PUD {
    pub const SPAN_BITS: usize = cap_type::PD::SPAN_BITS + (sys::seL4_PUDIndexBits as usize);
}

impl cap_type::PD {
    pub const SPAN_BITS: usize = cap_type::PT::SPAN_BITS + (sys::seL4_PageDirIndexBits as usize);
}

impl cap_type::PT {
    pub const SPAN_BITS: usize = FrameSize::Small.bits() + (sys::seL4_PageTableIndexBits as usize);
}
