use crate::bf::*;
use crate::c::*;

use sel4_config::sel4_cfg_match;

impl seL4_Fault {
    pub(crate) fn arch_get_with(label: seL4_Word, length: seL4_Word, f: impl Fn(core::ffi::c_ulong) -> seL4_Word) -> Option<Self> {
        Some({
            #[sel4_cfg_match]
            match label {
                seL4_Fault_tag::seL4_Fault_UnknownSyscall => {
                    assert!(length == seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_Length);
                    seL4_Fault_UnknownSyscall_Unpacked {
                        R0: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R0),
                        R1: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R1),
                        R2: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R2),
                        R3: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R3),
                        R4: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R4),
                        R5: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R5),
                        R6: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R6),
                        R7: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R7),
                        FaultIP: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_FaultIP),
                        SP: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_SP),
                        LR: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_LR),
                        CPSR: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_CPSR),
                        Syscall: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_Syscall),
                    }
                    .unsplay()
                }
                seL4_Fault_tag::seL4_Fault_UserException => {
                    assert!(length == seL4_UserException_Msg::seL4_UserException_Length);
                    seL4_Fault_UserException_Unpacked {
                        FaultIP: f(seL4_UserException_Msg::seL4_UserException_FaultIP),
                        Stack: f(seL4_UserException_Msg::seL4_UserException_SP),
                        CPSR: f(seL4_UserException_Msg::seL4_UserException_CPSR),
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
                _ => {
                    return None
                }
            }
        })
    }
}
