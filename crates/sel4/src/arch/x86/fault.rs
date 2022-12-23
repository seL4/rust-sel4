use crate::{declare_fault_newtype, sys};

declare_fault_newtype!(NullFault, sys::seL4_Fault_NullFault);
declare_fault_newtype!(CapFault, sys::seL4_Fault_CapFault);
declare_fault_newtype!(UnknownSyscall, sys::seL4_Fault_UnknownSyscall);
declare_fault_newtype!(UserException, sys::seL4_Fault_UserException);
declare_fault_newtype!(VMFault, sys::seL4_Fault_VMFault);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {
    NullFault(NullFault),
    CapFault(CapFault),
    UnknownSyscall(UnknownSyscall),
    UserException(UserException),
    VMFault(VMFault),
}

impl Fault {
    pub fn from_sys(raw: sys::seL4_Fault) -> Self {
        match raw.splay() {
            sys::seL4_Fault_Splayed::NullFault(inner) => {
                Self::NullFault(NullFault::from_inner(inner))
            }
            sys::seL4_Fault_Splayed::CapFault(inner) => Self::CapFault(CapFault::from_inner(inner)),
            sys::seL4_Fault_Splayed::UnknownSyscall(inner) => {
                Self::UnknownSyscall(UnknownSyscall::from_inner(inner))
            }
            sys::seL4_Fault_Splayed::UserException(inner) => {
                Self::UserException(UserException::from_inner(inner))
            }
            sys::seL4_Fault_Splayed::VMFault(inner) => Self::VMFault(VMFault::from_inner(inner)),
        }
    }
}
