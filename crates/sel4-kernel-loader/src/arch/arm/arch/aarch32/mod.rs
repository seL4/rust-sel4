//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::asm;
use core::mem;

use sel4_kernel_loader_payload_types::ArchivedPayloadInfo;

use crate::{arch::Arch, main, secondary_main};

pub(crate) mod drivers;

#[unsafe(no_mangle)]
extern "C" fn arch_main() -> ! {
    main(())
}

#[unsafe(no_mangle)]
extern "C" fn arch_secondary_main() -> ! {
    secondary_main(())
}

unsafe extern "C" {
    fn switch_translation_tables();
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
        _core_id: usize,
        payload_info: &ArchivedPayloadInfo,
        _per_core: Self::PerCore,
    ) -> ! {
        let kernel_entry =
            unsafe { mem::transmute::<usize, KernelEntry>(payload_info.kernel_entry.to_usize()) };

        let (dtb_addr_p, dtb_size) = match payload_info.dtb.as_ref() {
            Some(dtb) => (dtb.addr_p.to_usize(), dtb.size.to_usize()),
            None => (0, 0),
        };

        unsafe {
            switch_translation_tables();
        }

        (kernel_entry)(
            payload_info.user_image.ui_p_reg_start.to_usize(),
            payload_info.user_image.ui_p_reg_end.to_usize(),
            payload_info.user_image.pv_offset.to_usize(),
            payload_info.user_image.v_entry.to_usize(),
            dtb_addr_p,
            dtb_size,
        )
    }
}

type KernelEntry = extern "C" fn(
    ui_p_reg_start: usize,
    ui_p_reg_end: usize,
    pv_offset: usize,
    v_entry: usize,
    dtb_addr_p: usize,
    dtb_size: usize,
) -> !;

#[inline(never)]
pub(crate) unsafe fn reset_cntvoff() {
    unsafe {
        asm!("mcrr p15, 4, {val}, {val}, c14", val = in(reg) 0);
    }
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
