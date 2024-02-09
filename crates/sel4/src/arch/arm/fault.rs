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
declare_fault_newtype!(VmFault, sys::seL4_Fault_VMFault);

#[sel4_cfg(KERNEL_MCS)]
declare_fault_newtype!(Timeout, sys::seL4_Fault_Timeout);

sel4_cfg_if! {
    if #[cfg(ARM_HYPERVISOR_SUPPORT)] {
        declare_fault_newtype!(VGicMaintenance, sys::seL4_Fault_VGICMaintenance);
        declare_fault_newtype!(VCpuFault, sys::seL4_Fault_VCPUFault);
        declare_fault_newtype!(VPpiEvent, sys::seL4_Fault_VPPIEvent);
    }
}

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
    #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
    VGicMaintenance(VGicMaintenance),
    #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
    VCpuFault(VCpuFault),
    #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
    VPpiEvent(VPpiEvent),
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
                #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
                sys::seL4_Fault_Splayed::VGICMaintenance(inner) => {
                    Self::VGicMaintenance(VGicMaintenance::from_inner(inner))
                }
                #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
                sys::seL4_Fault_Splayed::VCPUFault(inner) => {
                    Self::VCpuFault(VCpuFault::from_inner(inner))
                }
                #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
                sys::seL4_Fault_Splayed::VPPIEvent(inner) => {
                    Self::VPpiEvent(VPpiEvent::from_inner(inner))
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

impl VmFault {
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
impl VGicMaintenance {
    pub fn idx(&self) -> Option<Word> {
        match self.inner().get_IDX() {
            Word::MAX => None,
            idx => Some(idx),
        }
    }
}

#[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
impl VCpuFault {
    pub fn hsr(&self) -> Word {
        self.inner().get_HSR()
    }
}

#[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
impl VPpiEvent {
    pub fn irq(&self) -> Word {
        self.inner().get_irq()
    }
}
