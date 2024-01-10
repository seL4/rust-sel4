//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use crate::bf::*;
use crate::c::*;

use sel4_config::sel4_cfg_match;

impl seL4_Fault {
    pub(crate) fn arch_get_with(
        label: seL4_Word,
        length: seL4_Word,
        f: impl Fn(core::ffi::c_ulong) -> seL4_Word,
    ) -> Option<Self> {
        Some({
            #[sel4_cfg_match]
            match label {
                seL4_Fault_tag::seL4_Fault_UnknownSyscall => {
                    assert!(length == seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_Length);
                    seL4_Fault_UnknownSyscall_Unpacked {
                        X0: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_X0),
                        X1: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_X1),
                        X2: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_X2),
                        X3: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_X3),
                        X4: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_X4),
                        X5: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_X5),
                        X6: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_X6),
                        X7: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_X7),
                        FaultIP: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_FaultIP),
                        SP: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_SP),
                        LR: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_LR),
                        SPSR: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_SPSR),
                        Syscall: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_Syscall),
                    }
                    .unsplay()
                }
                seL4_Fault_tag::seL4_Fault_UserException => {
                    assert!(length == seL4_UserException_Msg::seL4_UserException_Length);
                    seL4_Fault_UserException_Unpacked {
                        FaultIP: f(seL4_UserException_Msg::seL4_UserException_FaultIP),
                        Stack: f(seL4_UserException_Msg::seL4_UserException_SP),
                        SPSR: f(seL4_UserException_Msg::seL4_UserException_SPSR),
                        Number: f(seL4_UserException_Msg::seL4_UserException_Number),
                        Code: f(seL4_UserException_Msg::seL4_UserException_Code),
                    }
                    .unsplay()
                }
                seL4_Fault_tag::seL4_Fault_VMFault => {
                    assert!(length == seL4_VMFault_Msg::seL4_VMFault_Length);
                    seL4_Fault_VMFault_Unpacked {
                        IP: f(seL4_VMFault_Msg::seL4_VMFault_IP),
                        Addr: f(seL4_VMFault_Msg::seL4_VMFault_Addr),
                        PrefetchFault: f(seL4_VMFault_Msg::seL4_VMFault_PrefetchFault),
                        FSR: f(seL4_VMFault_Msg::seL4_VMFault_FSR),
                    }
                    .unsplay()
                }
                #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
                seL4_Fault_tag::seL4_Fault_VGICMaintenance => {
                    assert!(length == seL4_VGICMaintenance_Msg::seL4_VGICMaintenance_Length);
                    seL4_Fault_VGICMaintenance_Unpacked {
                        IDX: f(seL4_VGICMaintenance_Msg::seL4_VGICMaintenance_IDX),
                    }
                    .unsplay()
                }
                #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
                seL4_Fault_tag::seL4_Fault_VCPUFault => {
                    assert!(length == seL4_VCPUFault_Msg::seL4_VCPUFault_Length);
                    seL4_Fault_VCPUFault_Unpacked {
                        HSR: f(seL4_VCPUFault_Msg::seL4_VCPUFault_HSR),
                    }
                    .unsplay()
                }
                #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
                seL4_Fault_tag::seL4_Fault_VPPIEvent => {
                    // TODO
                    // assert!(length == seL4_VPPIEvent_Msg::seL4_VPPIEvent_Length);
                    seL4_Fault_VPPIEvent_Unpacked {
                        irq: f(seL4_VPPIEvent_Msg::seL4_VPPIEvent_IRQ),
                    }
                    .unsplay()
                }
                _ => return None,
            }
        })
    }
}
