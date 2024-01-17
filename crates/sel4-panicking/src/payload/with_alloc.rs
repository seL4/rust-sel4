//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::boxed::Box;
use core::any::{Any, TypeId};
use core::convert::AsRef;

use super::UpcastIntoPayload;

pub struct Payload(Box<dyn Any + Send + 'static>);

impl Payload {
    fn new(inner: Box<dyn Any + Send + 'static>) -> Self {
        Self(inner)
    }

    pub fn inner(&self) -> &(dyn Any + Send + 'static) {
        Box::as_ref(&self.0)
    }

    pub fn into_inner(self) -> Box<dyn Any + Send + 'static> {
        self.0
    }

    pub fn type_id(&self) -> TypeId {
        (*self.0).type_id()
    }

    pub fn downcast<T: Sized + 'static>(self) -> Result<T, Self> {
        match self.into_inner().downcast() {
            Ok(val) => Ok(*val),
            Err(orig) => Err(Self::new(orig)),
        }
    }
}

impl<T: Any + Send + 'static> UpcastIntoPayload for T {
    fn upcast_into_payload(self) -> Payload {
        Payload::new(Box::new(self))
    }
}
