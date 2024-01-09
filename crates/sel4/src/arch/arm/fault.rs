//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2020 Arm Limited
//
// SPDX-License-Identifier: MIT
//

use sel4_config::{sel4_cfg, sel4_cfg_enum, sel4_cfg_if, sel4_cfg_wrap_match};

use crate::{declare_fault_newtype, sys, Word};

declare_fault_newtype!(NullFault, sys::seL4_Fault_NullFault);
declare_fault_newtype!(CapFault, sys::seL4_Fault_CapFault);
declare_fault_newtype!(UnknownSyscall, sys::seL4_Fault_UnknownSyscall);
declare_fault_newtype!(UserException, sys::seL4_Fault_UserException);
declare_fault_newtype!(VMFault, sys::seL4_Fault_VMFault);

#[sel4_cfg(KERNEL_MCS)]
declare_fault_newtype!(Timeout, sys::seL4_Fault_Timeout);

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
    #[sel4_cfg(KERNEL_MCS)]
    Timeout(Timeout),
    #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
    VGICMaintenance(VGICMaintenance),
    #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
    VCPUFault(VCPUFault),
    #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
    VPPIEvent(VPPIEvent),
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
                sys::seL4_Fault_Splayed::VMFault(inner) => Self::VMFault(VMFault::from_inner(inner)),
                #[sel4_cfg(KERNEL_MCS)]
                sys::seL4_Fault_Splayed::Timeout(inner) => Self::Timeout(Timeout::from_inner(inner)),
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
}

impl CapFault {
    // TODO
}

impl UnknownSyscall {
    pub fn fault_ip(&self) -> Word {
        self.inner().get_FaultIP()
    }

    pub fn sp(&self) -> Word {
        self.inner().get_SP()
    }

    pub fn lr(&self) -> Word {
        self.inner().get_LR()
    }

    pub fn syscall(&self) -> Word {
        self.inner().get_Syscall()
    }
}

impl UserException {
    // TODO
}

impl VMFault {
    pub fn ip(&self) -> Word {
        self.inner().get_IP()
    }

    pub fn addr(&self) -> Word {
        self.inner().get_Addr()
    }

    pub fn is_prefetch(&self) -> bool {
        self.inner().get_PrefetchFault() != 0
    }

    pub fn fsr(&self) -> Word {
        self.inner().get_FSR()
    }
}

#[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
impl VGICMaintenance {
    pub fn idx(&self) -> Option<Word> {
        match self.inner().get_IDX() {
            Word::MAX => None,
            idx => Some(idx),
        }
    }
}

#[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
impl VCPUFault {
    pub fn hsr(&self) -> Word {
        self.inner().get_HSR()
    }
}

#[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
impl VPPIEvent {
    pub fn irq(&self) -> Word {
        self.inner().get_irq()
    }
}
