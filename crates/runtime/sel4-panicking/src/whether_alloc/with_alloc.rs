use alloc::boxed::Box;
use core::any::Any;

pub struct Payload(pub Option<Box<dyn Any + Send>>);

impl Payload {
    pub(crate) fn empty() -> Self {
        Self(None)
    }
}
