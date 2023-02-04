#[cfg(feature = "alloc")]
use alloc::string::String;

pub trait ObjectName {
    fn object_name(&self) -> Option<&str>;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Unnamed;

impl ObjectName for Unnamed {
    fn object_name(&self) -> Option<&str> {
        None
    }
}

impl ObjectName for str {
    fn object_name(&self) -> Option<&str> {
        Some(self)
    }
}

#[cfg(feature = "alloc")]
impl ObjectName for String {
    fn object_name(&self) -> Option<&str> {
        Some(self)
    }
}

impl<T: ObjectName> ObjectName for Option<T> {
    fn object_name(&self) -> Option<&str> {
        self.as_ref().and_then(ObjectName::object_name)
    }
}

impl<T: ObjectName + ?Sized> ObjectName for &T {
    fn object_name(&self) -> Option<&str> {
        <T as ObjectName>::object_name(self)
    }
}
