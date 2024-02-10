//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_config::sel4_cfg;

use super::get_ipc_buffer;
use crate::{seL4_Fault, seL4_Fault_tag, seL4_MessageInfo, wrappers::seL4_MessageInfo_get_label};

#[sel4_cfg(KERNEL_MCS)]
use core::mem;

#[sel4_cfg(KERNEL_MCS)]
use crate::{
    seL4_UserContext, seL4_Word,
    wrappers::{seL4_MessageInfo_new, seL4_SetMR},
};

#[no_mangle]
pub extern "C" fn seL4_getFault(tag: seL4_MessageInfo) -> seL4_Fault {
    seL4_Fault::get_from_ipc_buffer(&tag, get_ipc_buffer())
}

#[no_mangle]
pub extern "C" fn seL4_isVMFault_tag(tag: seL4_MessageInfo) -> bool {
    seL4_MessageInfo_get_label(tag) == seL4_Fault_tag::seL4_Fault_VMFault
}

#[sel4_cfg(KERNEL_MCS)]
#[no_mangle]
pub extern "C" fn seL4_isTimeoutFault_tag(tag: seL4_MessageInfo) -> bool {
    seL4_MessageInfo_get_label(tag) == seL4_Fault_tag::seL4_Fault_Timeout
}

#[sel4_cfg(KERNEL_MCS)]
#[no_mangle]
pub extern "C" fn seL4_TimeoutReply_new(
    resume: bool,
    regs: seL4_UserContext,
    length: seL4_Word,
) -> seL4_MessageInfo {
    let info = seL4_MessageInfo_new((!resume).into(), 0, 0, length);
    let regs_arr = unsafe {
        mem::transmute::<
            _,
            [seL4_Word; mem::size_of::<seL4_UserContext>() / mem::size_of::<seL4_Word>()],
        >(regs)
    };
    for i in 0usize..length.try_into().unwrap() {
        seL4_SetMR(i.try_into().unwrap(), regs_arr[i]);
    }
    info
}
