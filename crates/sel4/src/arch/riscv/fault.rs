use crate::sys;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {}

impl Fault {
    pub fn from_sys(raw: sys::seL4_Fault) -> Self {
        match raw.splay() {
            _ => unimplemented!(),
        }
    }
}
