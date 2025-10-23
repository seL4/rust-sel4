//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::asm;
use core::mem;

use riscv::register::satp;

use sel4_config::sel4_cfg_if;
use sel4_kernel_loader_payload_types::PayloadInfo;

use crate::{
    arch::Arch,
    main, secondary_main,
    this_image::page_tables::kernel::{
        kernel_boot_level_0_table, kernel_boot_level_0_table_access,
    },
};

pub(crate) struct PerCoreImpl {
    hart_id: usize,
}

#[unsafe(no_mangle)]
extern "C" fn arch_main(hart_id: usize) -> ! {
    main(PerCoreImpl { hart_id })
}

#[unsafe(no_mangle)]
extern "C" fn arch_secondary_main(hart_id: usize) -> ! {
    secondary_main(PerCoreImpl { hart_id })
}

pub(crate) enum ArchImpl {}

impl Arch for ArchImpl {
    type PerCore = PerCoreImpl;

    fn init() {
        unsafe {
            kernel_boot_level_0_table_access.finish_for_riscv();
        }
    }

    fn idle() -> ! {
        loop {
            unsafe {
                asm!("wfi");
            }
        }
    }

    #[allow(unused_variables)]
    fn enter_kernel(
        core_id: usize,
        payload_info: &PayloadInfo<usize>,
        per_core: Self::PerCore,
    ) -> ! {
        let kernel_entry =
            unsafe { mem::transmute::<usize, KernelEntry>(payload_info.kernel_image.virt_entry) };

        let ui_p_reg_start = payload_info.user_image.phys_addr_range.start;
        let ui_p_reg_end = payload_info.user_image.phys_addr_range.end;
        let pv_offset = 0_usize.wrapping_sub(payload_info.user_image.phys_to_virt_offset) as isize;
        let v_entry = payload_info.user_image.virt_entry;

        let (dtb_addr_p, dtb_size) = match &payload_info.fdt_phys_addr_range {
            Some(region) => (region.start, region.len()),
            None => (0, 0),
        };

        let hart_id = per_core.hart_id;

        switch_page_tables();

        sel4_cfg_if! {
            if #[sel4_cfg(MAX_NUM_NODES = "1")] {
                (kernel_entry)(
                    ui_p_reg_start,
                    ui_p_reg_end,
                    pv_offset,
                    v_entry,
                    dtb_addr_p,
                    dtb_size,
                )
            } else {
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
        }
    }
}

sel4_cfg_if! {
    if #[sel4_cfg(MAX_NUM_NODES = "1")] {
        type KernelEntry = extern "C" fn(
            ui_p_reg_start: usize,
            ui_p_reg_end: usize,
            pv_offset: isize,
            v_entry: usize,
            dtb_addr_p: usize,
            dtb_size: usize,
        ) -> !;
    } else {
        type KernelEntry = extern "C" fn(
            ui_p_reg_start: usize,
            ui_p_reg_end: usize,
            pv_offset: isize,
            v_entry: usize,
            dtb_addr_p: usize,
            dtb_size: usize,
            hart_id: usize,
            core_id: usize,
        ) -> !;
    }
}

fn switch_page_tables() {
    #[cfg(target_pointer_width = "32")]
    const MODE: satp::Mode = satp::Mode::Sv32;

    #[cfg(target_pointer_width = "64")]
    const MODE: satp::Mode = satp::Mode::Sv39;

    unsafe {
        let ppn = kernel_boot_level_0_table.value() as usize >> 12;
        asm!("sfence.vma", options(nostack));
        satp::set(MODE, 0, ppn);
        asm!("fence.i", options(nostack));
    }
}
