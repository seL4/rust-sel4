pub const GRANULE_SIZE: FrameSize = FrameSize::Granule;

impl FrameSize {
    pub const fn bits(self) -> usize {
        match self {
            FrameSize::Granule => 12, // [TODO] rename
        }
    }

    pub const fn bytes(self) -> usize {
        1 << self.bits()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum FrameSize {
    Granule,
}
