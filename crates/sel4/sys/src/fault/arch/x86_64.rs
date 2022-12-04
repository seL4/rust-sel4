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
                        RAX: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_RAX),
                        RBX: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_RBX),
                        RCX: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_RCX),
                        RDX: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_RDX),
                        RSI: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_RSI),
                        RDI: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_RDI),
                        RBP: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_RBP),
                        R8: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R8),
                        R9: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R9),
                        R10: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R10),
                        R11: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R11),
                        R12: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R12),
                        R13: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R13),
                        R14: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R14),
                        R15: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_R15),
                        FaultIP: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_FaultIP),
                        RSP: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_SP),
                        FLAGS: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_FLAGS),
                        Syscall: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_Syscall),
                    }
                    .unsplay()
                }
                seL4_Fault_tag::seL4_Fault_UserException => {
                    assert!(length == seL4_UserException_Msg::seL4_UserException_Length);
                    seL4_Fault_UserException_Unpacked {
                        FaultIP: f(seL4_UserException_Msg::seL4_UserException_FaultIP),
                        Stack: f(seL4_UserException_Msg::seL4_UserException_SP),
                        FLAGS: f(seL4_UserException_Msg::seL4_UserException_FLAGS),
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
                _ => {
                    return None
                }
            }
        })
    }
}
