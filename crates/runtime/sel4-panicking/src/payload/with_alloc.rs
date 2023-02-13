use alloc::boxed::Box;
use core::any::Any;
use core::convert::AsRef;

pub struct Payload(Box<dyn 'static + Any + Send>);

impl Payload {
    pub fn new(inner: impl 'static + Any + Send) -> Self {
        Self(Box::new(inner))
    }

    pub fn inner(&self) -> &(dyn 'static + Any + Send) {
        Box::as_ref(&self.0)
    }

    pub fn into_inner(self) -> Box<dyn 'static + Any + Send> {
        self.0
    }
}

pub trait IntoPayload {
    fn into_payload(self) -> Payload;
}

pub trait TryFromPayload {
    fn try_from_payload(payload: &Payload) -> Option<&Self>;
}

impl<T: 'static + Any + Send> IntoPayload for T {
    fn into_payload(self) -> Payload {
        Payload::new(self)
    }
}

impl<T: 'static + Any + Send> TryFromPayload for T {
    fn try_from_payload(payload: &Payload) -> Option<&Self> {
        payload.inner().downcast_ref()
    }
}
