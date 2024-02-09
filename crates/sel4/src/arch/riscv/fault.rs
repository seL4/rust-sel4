//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use sel4_config::{sel4_cfg, sel4_cfg_enum, sel4_cfg_wrap_match};

use crate::{declare_fault_newtype, sys};

declare_fault_newtype!(NullFault, sys::seL4_Fault_NullFault);
declare_fault_newtype!(CapFault, sys::seL4_Fault_CapFault);
declare_fault_newtype!(UnknownSyscall, sys::seL4_Fault_UnknownSyscall);
declare_fault_newtype!(UserException, sys::seL4_Fault_UserException);
declare_fault_newtype!(VmFault, sys::seL4_Fault_VMFault);

#[sel4_cfg(KERNEL_MCS)]
declare_fault_newtype!(Timeout, sys::seL4_Fault_Timeout);

#[sel4_cfg_enum]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {
    NullFault(NullFault),
    CapFault(CapFault),
    UnknownSyscall(UnknownSyscall),
    UserException(UserException),
    VmFault(VmFault),
    #[sel4_cfg(KERNEL_MCS)]
    Timeout(Timeout),
}

impl Fault {
    pub fn from_sys(raw: sys::seL4_Fault) -> Self {
        sel4_cfg_wrap_match! {
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
                sys::seL4_Fault_Splayed::VMFault(inner) => Self::VmFault(VmFault::from_inner(inner)),
                #[sel4_cfg(KERNEL_MCS)]
                sys::seL4_Fault_Splayed::Timeout(inner) => Self::Timeout(Timeout::from_inner(inner)),
            }
        }
    }
}
