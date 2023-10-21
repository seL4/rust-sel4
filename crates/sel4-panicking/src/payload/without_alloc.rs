//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::any::{Any, TypeId};

use super::{FitsWithinSmallPayload, SmallPayloadValue, UpcastIntoPayload};

pub struct Payload {
    type_id: TypeId,
    value: SmallPayloadValue,
}

impl Payload {
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn downcast<T: FitsWithinSmallPayload + Copy + 'static>(self) -> Result<T, Self> {
        if self.type_id() == TypeId::of::<T>() {
            Ok(self.value.read())
        } else {
            Err(self)
        }
    }
}

impl<T: FitsWithinSmallPayload + Copy + Any> UpcastIntoPayload for T {
    fn upcast_into_payload(self) -> Payload {
        let type_id = self.type_id();
        Payload {
            type_id,
            value: SmallPayloadValue::write(&self),
        }
    }
}
