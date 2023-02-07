use core::borrow::{Borrow, BorrowMut};

use sel4::InitCSpaceSlot;

pub struct LoaderBuffers<T> {
    pub(crate) per_obj: T,
}

#[derive(Copy, Clone)]
pub struct PerObjectBuffer {
    pub(crate) orig_slot: Option<InitCSpaceSlot>,
}

#[allow(clippy::derivable_impls)] // until #![feature(derive_const)]
impl const Default for PerObjectBuffer {
    fn default() -> Self {
        Self { orig_slot: None }
    }
}

impl<T> LoaderBuffers<T> {
    pub const fn new(per_obj: T) -> Self {
        Self { per_obj }
    }
}

impl<T: Borrow<[PerObjectBuffer]>> LoaderBuffers<T> {
    pub fn per_obj(&self) -> &[PerObjectBuffer] {
        self.per_obj.borrow()
    }
}

impl<T: BorrowMut<[PerObjectBuffer]>> LoaderBuffers<T> {
    pub fn per_obj_mut(&mut self) -> &mut [PerObjectBuffer] {
        self.per_obj.borrow_mut()
    }
}
