//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::rc::Rc;
use alloc::sync::Arc;
use core::ops::Deref;

pub trait AbstractRcT {
    type Rc<T>: AbstractRc<T>;
}

pub struct RcT(());

impl AbstractRcT for RcT {
    type Rc<T> = Rc<T>;
}

pub struct ArcT(());

impl AbstractRcT for ArcT {
    type Rc<T> = Arc<T>;
}

pub trait AbstractRc<T>: Deref<Target = T> + From<T> + Clone {}

impl<T> AbstractRc<T> for Rc<T> {}

impl<T> AbstractRc<T> for Arc<T> {}
