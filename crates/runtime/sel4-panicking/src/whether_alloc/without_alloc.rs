use core::any::Any;

pub struct Payload(pub Option<Region>);

pub enum Region {
    Val(usize),
    Ref(&'static (dyn Any + Send)),
}

impl Payload {
    pub(crate) fn empty() -> Self {
        Self(None)
    }
}
