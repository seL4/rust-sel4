//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::any::{Any, TypeId};
use core::mem;
use core::ptr;
use core::slice;

use super::{check_small_payload_size, SmallPayload, UpcastIntoPayload, SMALL_PAYLOAD_MAX_SIZE};

pub struct Payload {
    type_id: TypeId,
    value: SmallPayloadValue,
}

impl Payload {
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn downcast<T: SmallPayload + Copy + 'static>(self) -> Result<T, Self> {
        if self.type_id() == TypeId::of::<T>() {
            Ok(self.value.read())
        } else {
            Err(self)
        }
    }
}

impl<T: SmallPayload + Copy + Any> UpcastIntoPayload for T {
    fn upcast_into_payload(self) -> Payload {
        let type_id = self.type_id();
        Payload {
            type_id,
            value: SmallPayloadValue::write(&self),
        }
    }
}

#[derive(Clone, Copy)]
struct SmallPayloadValue([u8; SMALL_PAYLOAD_MAX_SIZE]);

impl SmallPayloadValue {
    fn write<T: SmallPayload + Copy>(val: &T) -> Self {
        check_small_payload_size::<T>();
        let val_bytes =
            unsafe { slice::from_raw_parts(ptr::addr_of!(*val).cast::<u8>(), mem::size_of::<T>()) };
        let mut payload_arr = [0; SMALL_PAYLOAD_MAX_SIZE];
        payload_arr[..val_bytes.len()].copy_from_slice(val_bytes);
        Self(payload_arr)
    }

    fn read<T: SmallPayload + Copy>(&self) -> T {
        check_small_payload_size::<T>();
        unsafe { mem::transmute_copy(&self.0) }
    }
}
