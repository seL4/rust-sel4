//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_config::sel4_cfg;

use super::get_ipc_buffer;
use crate::{seL4_Word, SEL4_MAPPING_LOOKUP_LEVEL};

#[sel4_cfg(KERNEL_MCS)]
use crate::{
    seL4_Error, seL4_SchedContext, seL4_SchedContextFlag, seL4_SchedControl, seL4_Time,
    wrappers::get_ipc_buffer_mut,
};

#[no_mangle]
pub extern "C" fn seL4_MappingFailedLookupLevel() -> seL4_Word {
    get_ipc_buffer().get_mr(SEL4_MAPPING_LOOKUP_LEVEL.try_into().unwrap())
}

// alias
#[no_mangle]
#[sel4_cfg(KERNEL_MCS)]
pub extern "C" fn seL4_SchedControl_Configure(
    service: seL4_SchedControl,
    schedcontext: seL4_SchedContext,
    budget: seL4_Time,
    period: seL4_Time,
    extra_refills: seL4_Word,
    badge: seL4_Word,
) -> seL4_Error::Type {
    get_ipc_buffer_mut().seL4_SchedControl_ConfigureFlags(
        service,
        schedcontext,
        budget,
        period,
        extra_refills,
        badge,
        seL4_SchedContextFlag::seL4_SchedContext_NoFlag,
    )
}
