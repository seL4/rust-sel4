use core::convert::AsRef;
use core::fmt;

pub trait ObjectName: fmt::Display {
    fn object_name(&self) -> Option<&[u8]>;
}

impl<T: fmt::Display + AsRef<str>> ObjectName for T {
    fn object_name(&self) -> Option<&[u8]> {
        Some(self.as_ref().as_bytes())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Unnamed;

impl fmt::Display for Unnamed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(unnamed)")
    }
}

impl ObjectName for Unnamed {
    fn object_name(&self) -> Option<&[u8]> {
        None
    }
}
