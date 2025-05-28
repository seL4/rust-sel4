//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use crate::bf::*;
use crate::c::*;

use sel4_config::sel4_cfg_wrap_match;

mod arch;

impl seL4_Fault {
    pub fn get_from_ipc_buffer(info: &seL4_MessageInfo, ipcbuf: &seL4_IPCBuffer) -> Self {
        Self::get_with(info.get_label(), info.get_length(), |i| {
            ipcbuf.msg[i as usize]
        })
    }

    pub fn get_with(
        label: seL4_Word,
        length: seL4_Word,
        f: impl Fn(core::ffi::c_ulong) -> seL4_Word,
    ) -> Self {
        sel4_cfg_wrap_match! {
            match label {
                seL4_Fault_tag::seL4_Fault_NullFault => {
                    // TODO
                    // assert!(length == seL4_NullFault_Msg::seL4_NullFault_Length);
                    seL4_Fault_NullFault_Unpacked {}.unsplay()
                }
                seL4_Fault_tag::seL4_Fault_CapFault => {
                    // TODO
                    // assert!(length == seL4_CapFault_Msg::seL4_CapFault_Length);
                    seL4_Fault_CapFault_Unpacked {
                        IP: f(seL4_CapFault_Msg::seL4_CapFault_IP),
                        Addr: f(seL4_CapFault_Msg::seL4_CapFault_Addr),
                        InRecvPhase: f(seL4_CapFault_Msg::seL4_CapFault_InRecvPhase),
                        LookupFailureType: f(seL4_CapFault_Msg::seL4_CapFault_LookupFailureType),
                        MR4: f(seL4_CapFault_Msg::seL4_CapFault_BitsLeft),
                        MR5: f(seL4_CapFault_Msg::seL4_CapFault_GuardMismatch_GuardFound),
                        MR6: f(seL4_CapFault_Msg::seL4_CapFault_GuardMismatch_BitsFound),
                    }
                    .unsplay()
                }
                #[sel4_cfg(KERNEL_MCS)]
                seL4_Fault_tag::seL4_Fault_Timeout => {
                    assert!(length == seL4_Timeout_Msg::seL4_Timeout_Length.into());
                    seL4_Fault_Timeout_Unpacked {
                        data: f(seL4_Timeout_Msg::seL4_Timeout_Data.into()),
                        consumed: f(seL4_Timeout_Msg::seL4_Timeout_Consumed.into()),
                    }
                    .unsplay()
                }
                _ => {
                    match Self::arch_get_with(label, length, f) {
                    Some(fault) => fault,
                    None => panic!("Unparsed fault label: {label}"),
                }},
            }
        }
    }
}
