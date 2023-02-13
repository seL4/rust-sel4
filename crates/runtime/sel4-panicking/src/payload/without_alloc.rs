use core::any::{Any, TypeId};

pub struct Payload {
    type_id: TypeId,
    value: PayloadValue,
}

pub const PAYLOAD_VALUE_SIZE: usize = 32;

pub type PayloadValue = [u8; PAYLOAD_VALUE_SIZE];

pub trait IntoPayload {
    fn into_payload(self) -> Payload;
}

pub trait TryFromPayload: Sized {
    fn try_from_payload(payload: &Payload) -> Option<Self>;
}

impl<T: IntoPayloadValue + Any> IntoPayload for T {
    fn into_payload(self) -> Payload {
        let type_id = self.type_id();
        Payload {
            type_id,
            value: self.into_payload_value(),
        }
    }
}

impl<T: FromPayloadValue + Any> TryFromPayload for T {
    fn try_from_payload(payload: &Payload) -> Option<Self> {
        if payload.type_id == TypeId::of::<T>() {
            Some(T::from_payload_value(&payload.value))
        } else {
            None
        }
    }
}

pub unsafe trait IntoPayloadValue: Copy {
    fn into_payload_value(self) -> PayloadValue;
}

pub unsafe trait FromPayloadValue: Copy {
    fn from_payload_value(payload_value: &PayloadValue) -> Self;
}

unsafe impl IntoPayloadValue for () {
    fn into_payload_value(self) -> PayloadValue {
        Default::default()
    }
}

unsafe impl FromPayloadValue for () {
    fn from_payload_value(_payload_value: &PayloadValue) -> Self {
        Default::default()
    }
}
