use sel4_config::{sel4_cfg, sel4_cfg_enum, sel4_cfg_match};

use crate::sys;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NullFault(pub sys::seL4_Fault_NullFault);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapFault(pub sys::seL4_Fault_CapFault);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownSyscall(pub sys::seL4_Fault_UnknownSyscall);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserException(pub sys::seL4_Fault_UserException);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VMFault(pub sys::seL4_Fault_VMFault);

#[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VGICMaintenance(pub sys::seL4_Fault_VGICMaintenance);

#[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VCPUFault(pub sys::seL4_Fault_VCPUFault);

#[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VPPIEvent(pub sys::seL4_Fault_VPPIEvent);

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
            sys::seL4_Fault_Splayed::NullFault(inner) => Self::NullFault(NullFault(inner)),
            sys::seL4_Fault_Splayed::CapFault(inner) => Self::CapFault(CapFault(inner)),
            sys::seL4_Fault_Splayed::UnknownSyscall(inner) => {
                Self::UnknownSyscall(UnknownSyscall(inner))
            }
            sys::seL4_Fault_Splayed::UserException(inner) => {
                Self::UserException(UserException(inner))
            }
            sys::seL4_Fault_Splayed::VMFault(inner) => Self::VMFault(VMFault(inner)),
            #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
            sys::seL4_Fault_Splayed::VGICMaintenance(inner) => {
                Self::VGICMaintenance(VGICMaintenance(inner))
            }
            #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
            sys::seL4_Fault_Splayed::VCPUFault(inner) => Self::VCPUFault(VCPUFault(inner)),
            #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
            sys::seL4_Fault_Splayed::VPPIEvent(inner) => Self::VPPIEvent(VPPIEvent(inner)),
        }
    }
}
