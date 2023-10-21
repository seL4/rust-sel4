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
    arch::Arch, main, secondary_main, this_image::page_tables::kernel::kernel_boot_level_0_table,
};

pub(crate) struct PerCoreImpl {
    hart_id: usize,
}

#[no_mangle]
extern "C" fn arch_main(hart_id: usize) -> ! {
    main(PerCoreImpl { hart_id })
}

#[no_mangle]
extern "C" fn arch_secondary_main(hart_id: usize) -> ! {
    secondary_main(PerCoreImpl { hart_id })
}

pub(crate) enum ArchImpl {}

impl Arch for ArchImpl {
    type PerCore = PerCoreImpl;

    fn init() {
        unsafe {
            kernel_boot_level_0_table.finish();
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
            if #[cfg(MAX_NUM_NODES = "1")] {
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
    if #[cfg(MAX_NUM_NODES = "1")] {
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
    unsafe fn by_ptr_width(ppn: usize) {
        use core::arch::riscv32::{fence_i, sfence_vma_all};

        sfence_vma_all();
        satp::set(satp::Mode::Sv32, 0, ppn);
        fence_i();
    }

    #[cfg(target_pointer_width = "64")]
    unsafe fn by_ptr_width(ppn: usize) {
        use core::arch::riscv64::{fence_i, sfence_vma_all};

        sfence_vma_all();
        satp::set(satp::Mode::Sv39, 0, ppn);
        fence_i();
    }

    unsafe {
        let ppn = kernel_boot_level_0_table.root() as usize >> 12;
        by_ptr_width(ppn);
    }
}
