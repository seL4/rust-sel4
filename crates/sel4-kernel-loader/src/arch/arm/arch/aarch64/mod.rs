//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::asm;
use core::mem;

use aarch64_cpu::registers::{CurrentEL, Readable};

use sel4_kernel_loader_payload_types::PayloadInfo;

use crate::{arch::Arch, main, secondary_main};

pub(crate) mod drivers;
pub(crate) mod exception_handler;

extern "C" {
    fn switch_translation_tables_el2();
}

#[unsafe(no_mangle)]
extern "C" fn arch_main() -> ! {
    main(())
}

#[unsafe(no_mangle)]
extern "C" fn arch_secondary_main() -> ! {
    secondary_main(())
}

pub(crate) enum ArchImpl {}

impl Arch for ArchImpl {
    type PerCore = ();

    fn idle() -> ! {
        loop {
            unsafe {
                asm!("wfe");
            }
        }
    }

    fn enter_kernel(
        core_id: usize,
        payload_info: &PayloadInfo<usize>,
        _per_core: Self::PerCore,
    ) -> ! {
        let kernel_entry =
            unsafe { mem::transmute::<usize, KernelEntry>(payload_info.kernel_image.virt_entry) };

        let (dtb_addr_p, dtb_size) = match &payload_info.fdt_phys_addr_range {
            Some(region) => (region.start, region.len()),
            None => (0, 0),
        };

        let current_el = get_current_el();
        assert!(current_el == Some(CurrentEL::EL::Value::EL2));

        unsafe {
            set_tpidr(core_id);
        }

        unsafe {
            switch_translation_tables_el2();
        }

        (kernel_entry)(
            payload_info.user_image.phys_addr_range.start,
            payload_info.user_image.phys_addr_range.end,
            0_usize.wrapping_sub(payload_info.user_image.phys_to_virt_offset) as isize,
            payload_info.user_image.virt_entry,
            dtb_addr_p,
            dtb_size,
        )
    }
}

type KernelEntry = extern "C" fn(
    ui_p_reg_start: usize,
    ui_p_reg_end: usize,
    pv_offset: isize,
    v_entry: usize,
    dtb_addr_p: usize,
    dtb_size: usize,
) -> !;

fn get_current_el() -> Option<CurrentEL::EL::Value> {
    CurrentEL.read_as_enum(CurrentEL::EL)
}

#[inline(never)] // never inline to work around issues with optimizer
unsafe fn set_tpidr(tpidr: usize) {
    asm!("msr tpidr_el1, {tpidr}", tpidr = in(reg) tpidr);
}

#[inline(never)]
pub(crate) unsafe fn reset_cntvoff() {
    asm!("msr cntvoff_el2, xzr");
}
