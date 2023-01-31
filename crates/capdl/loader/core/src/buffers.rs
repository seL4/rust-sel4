use sel4::InitCSpaceSlot;

pub struct LoaderBuffers<const N: usize> {
    pub(crate) per_obj: [PerObjectBuffer; N],
}

pub(crate) struct PerObjectBuffer {
    pub(crate) orig_slot: Option<InitCSpaceSlot>,
}

impl<const N: usize> LoaderBuffers<N> {
    pub const fn new() -> Self {
        const INIT: PerObjectBuffer = PerObjectBuffer { orig_slot: None };
        Self { per_obj: [INIT; N] }
    }
}
