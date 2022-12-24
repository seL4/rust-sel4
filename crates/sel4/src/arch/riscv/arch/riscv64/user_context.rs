use crate::{newtype_methods, sys};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UserContext(sys::seL4_UserContext);

impl UserContext {
    newtype_methods!(sys::seL4_UserContext);
}
