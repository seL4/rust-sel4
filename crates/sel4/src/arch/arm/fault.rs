use sel4_config::{sel4_cfg_enum, sel4_cfg_if, sel4_cfg_match};

use crate::{declare_fault_newtype, sys};

declare_fault_newtype!(NullFault, sys::seL4_Fault_NullFault);
declare_fault_newtype!(CapFault, sys::seL4_Fault_CapFault);
declare_fault_newtype!(UnknownSyscall, sys::seL4_Fault_UnknownSyscall);
declare_fault_newtype!(UserException, sys::seL4_Fault_UserException);
declare_fault_newtype!(VMFault, sys::seL4_Fault_VMFault);

sel4_cfg_if! {
    if #[cfg(ARM_HYPERVISOR_SUPPORT)] {
        declare_fault_newtype!(VGICMaintenance, sys::seL4_Fault_VGICMaintenance);
        declare_fault_newtype!(VCPUFault, sys::seL4_Fault_VCPUFault);
        declare_fault_newtype!(VPPIEvent, sys::seL4_Fault_VPPIEvent);
    }
}

#[sel4_cfg_enum]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {
    NullFault(NullFault),
    CapFault(CapFault),
    UnknownSyscall(UnknownSyscall),
    UserException(UserException),
    VMFault(VMFault),
    #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
    VGICMaintenance(VGICMaintenance),
    #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
    VCPUFault(VCPUFault),
    #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
    VPPIEvent(VPPIEvent),
}

impl Fault {
    pub fn from_sys(raw: sys::seL4_Fault) -> Self {
        #[sel4_cfg_match]
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
            #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
            sys::seL4_Fault_Splayed::VGICMaintenance(inner) => {
                Self::VGICMaintenance(VGICMaintenance::from_inner(inner))
            }
            #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
            sys::seL4_Fault_Splayed::VCPUFault(inner) => {
                Self::VCPUFault(VCPUFault::from_inner(inner))
            }
            #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
            sys::seL4_Fault_Splayed::VPPIEvent(inner) => {
                Self::VPPIEvent(VPPIEvent::from_inner(inner))
            }
        }
    }
}
