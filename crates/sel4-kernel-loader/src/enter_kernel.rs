//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::mem;

use sel4_config::sel4_cfg_if;
use sel4_kernel_loader_payload_types::ArchivedPayloadInfo;

#[allow(unused_imports)]
use crate::arch::{Arch, ArchImpl};

#[allow(unused_variables)]
pub(crate) fn mk_enter_kernel(payload_info: &ArchivedPayloadInfo, core_id: usize) -> impl Fn() {
    let kernel_entry =
        unsafe { mem::transmute::<usize, KernelEntry>(payload_info.kernel_entry.to_usize()) };

    let ui_p_reg_start = payload_info.user_image.ui_p_reg_start.to_usize();
    let ui_p_reg_end = payload_info.user_image.ui_p_reg_end.to_usize();
    let pv_offset = payload_info.user_image.pv_offset.to_usize();
    let v_entry = payload_info.user_image.v_entry.to_usize();
    let dtb_addr_p = payload_info
        .dtb
        .as_ref()
        .map(|x| x.addr_p.to_usize())
        .unwrap_or(0);
    let dtb_size = payload_info
        .dtb
        .as_ref()
        .map(|x| x.size.to_usize())
        .unwrap_or(0);

    sel4_cfg_if! {
        if #[sel4_cfg(all(ARCH_RISCV, not(MAX_NUM_NODES = "1")))] {
            let hart_id = ArchImpl::logical_to_physical_core_id(core_id);
            move || {
                (kernel_entry)(
                    ui_p_reg_start,
                    ui_p_reg_end,
                    pv_offset,
                    v_entry,
                    dtb_addr_p,
                    dtb_size,
                    hart_id,
                    core_id,
                )
            }
        } else {
            move || {
                (kernel_entry)(
                    ui_p_reg_start,
                    ui_p_reg_end,
                    pv_offset,
                    v_entry,
                    dtb_addr_p,
                    dtb_size,
                )
            }
        }
    }
}

sel4_cfg_if! {
    if #[sel4_cfg(all(ARCH_RISCV, not(MAX_NUM_NODES = "1")))] {
        type KernelEntry = extern "C" fn(
            ui_p_reg_start: usize,
            ui_p_reg_end: usize,
            pv_offset: usize,
            v_entry: usize,
            dtb_addr_p: usize,
            dtb_size: usize,
            hart_id: usize,
            core_id: usize,
        ) -> !;
    } else {
        type KernelEntry = extern "C" fn(
            ui_p_reg_start: usize,
            ui_p_reg_end: usize,
            pv_offset: usize,
            v_entry: usize,
            dtb_addr_p: usize,
            dtb_size: usize,
        ) -> !;
    }
}
