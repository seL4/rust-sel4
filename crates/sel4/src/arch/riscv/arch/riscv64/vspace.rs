impl FrameSize {
    pub const fn bits(self) -> usize {
        match self {
            FrameSize::Granule => 12, // [TODO] rename
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum FrameSize {
    Granule,
}
