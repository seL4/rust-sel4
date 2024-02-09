//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::borrow::{Borrow, BorrowMut};

use sel4::init_thread::Slot;

pub struct InitializerBuffers<T> {
    pub(crate) per_obj: T,
}

#[derive(Copy, Clone)]
pub struct PerObjectBuffer {
    pub(crate) orig_slot: Option<Slot>,
}

#[allow(clippy::derivable_impls)] // until #![feature(derive_const)]
impl PerObjectBuffer {
    pub const fn const_default() -> Self {
        Self { orig_slot: None }
    }
}

impl<T> InitializerBuffers<T> {
    pub const fn new(per_obj: T) -> Self {
        Self { per_obj }
    }
}

impl<T: Borrow<[PerObjectBuffer]>> InitializerBuffers<T> {
    pub fn per_obj(&self) -> &[PerObjectBuffer] {
        self.per_obj.borrow()
    }
}

impl<T: BorrowMut<[PerObjectBuffer]>> InitializerBuffers<T> {
    pub fn per_obj_mut(&mut self) -> &mut [PerObjectBuffer] {
        self.per_obj.borrow_mut()
    }
}
