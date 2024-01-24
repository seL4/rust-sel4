//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::asm;
use core::mem;

use sel4_kernel_loader_payload_types::PayloadInfo;

use crate::{arch::Arch, main, secondary_main};

pub(crate) mod drivers;

#[no_mangle]
extern "C" fn arch_main() -> ! {
    main(())
}

#[no_mangle]
extern "C" fn arch_secondary_main() -> ! {
    secondary_main(())
}

extern "C" {
    fn switch_translation_tables();
}

pub(crate) enum ArchImpl {}

impl Arch for ArchImpl {
    type PerCore = ();

    fn init() {
        crate::fmt::debug_println_without_synchronization!("is_hyp_mode() -> {:?}", is_hyp_mode());
    }

    fn idle() -> ! {
        loop {
            unsafe {
                asm!("wfe");
            }
        }
    }

    fn enter_kernel(
        _core_id: usize,
        payload_info: &PayloadInfo<usize>,
        _per_core: Self::PerCore,
    ) -> ! {
        let kernel_entry =
            unsafe { mem::transmute::<usize, KernelEntry>(payload_info.kernel_image.virt_entry) };

        let (dtb_addr_p, dtb_size) = match &payload_info.fdt_phys_addr_range {
            Some(region) => (region.start, region.len()),
            None => (0, 0),
        };

        unsafe {
            switch_translation_tables();
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

#[inline(never)]
pub(crate) unsafe fn reset_cntvoff() {
    asm!("mcrr p15, 4, {val}, {val}, c14", val = in(reg) 0);
}

const CPSR_MODE_MASK: usize = 0x1f;
const CPSR_MODE_HYPERVISOR: usize = 0x1a;

fn is_hyp_mode() -> bool {
    let mut val: usize;
    unsafe {
        asm!("mrs {val}, cpsr", val = out(reg) val);
    }
    (val & CPSR_MODE_MASK) == CPSR_MODE_HYPERVISOR
}
