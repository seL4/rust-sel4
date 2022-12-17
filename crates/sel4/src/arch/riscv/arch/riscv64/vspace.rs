impl FrameSize {
    pub const fn bits(self) -> usize {
        match self {
            FrameSize::Small => 12,
        }
    }

    pub const fn bytes(self) -> usize {
        1 << self.bits()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum FrameSize {
    Small,
}
