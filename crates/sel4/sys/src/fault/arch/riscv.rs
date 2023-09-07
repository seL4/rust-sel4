use crate::bf::*;
use crate::c::*;

use sel4_config::sel4_cfg_match;

impl seL4_Fault {
    pub(crate) fn arch_get_with(label: seL4_Word, length: seL4_Word, f: impl Fn(core::ffi::c_ulong) -> seL4_Word) -> Option<Self> {
        let f = |i: core::ffi::c_uint| f(i.try_into().unwrap());
        let length: core::ffi::c_uint = length.try_into().unwrap();
        Some({
            #[sel4_cfg_match]
            match label {
                seL4_Fault_tag::seL4_Fault_UnknownSyscall => {
                    assert!(length == seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_Length);
                    seL4_Fault_UnknownSyscall_Unpacked {
                        FaultIP: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_FaultIP),
                        SP: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_SP),
                        RA: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_RA),
                        A0: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_A0),
                        A1: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_A1),
                        A2: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_A2),
                        A3: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_A3),
                        A4: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_A4),
                        A5: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_A5),
                        A6: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_A6),
                        Syscall: f(seL4_UnknownSyscall_Msg::seL4_UnknownSyscall_Syscall),
                    }
                    .unsplay()
                }
                seL4_Fault_tag::seL4_Fault_UserException => {
                    assert!(length == seL4_UserException_Msg::seL4_UserException_Length);
                    seL4_Fault_UserException_Unpacked {
                        FaultIP: f(seL4_UserException_Msg::seL4_UserException_FaultIP),
                        SP: f(seL4_UserException_Msg::seL4_UserException_SP),
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
