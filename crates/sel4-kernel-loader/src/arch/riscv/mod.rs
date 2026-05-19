//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use riscv::register::satp;

use sel4_config::sel4_cfg_usize;

use crate::{arch::Arch, main, secondary_main, this_image::kernel_boot_level_0_table};

#[unsafe(no_mangle)]
extern "C" fn arch_main(hart_id: usize) -> ! {
    main(hart_id)
}

#[unsafe(no_mangle)]
extern "C" fn arch_secondary_main(hart_id: usize) -> ! {
    secondary_main(hart_id)
}

pub(crate) enum ArchImpl {}

impl Arch for ArchImpl {
    fn physical_to_logical_core_id(physical_core_id: usize) -> Option<usize> {
        let logical_core_id = physical_core_id.checked_sub(sel4_cfg_usize!(FIRST_HART_ID))?;
        if logical_core_id < sel4_cfg_usize!(MAX_NUM_NODES) {
            Some(logical_core_id)
        } else {
            None
        }
    }

    fn logical_to_physical_core_id(logical_core_id: usize) -> usize {
        logical_core_id + sel4_cfg_usize!(FIRST_HART_ID)
    }

    fn idle() -> ! {
        loop {
            riscv::asm::wfi();
        }
    }

    fn prepare_to_enter_kernel(_core_id: usize) {
        switch_page_tables();
    }
}

fn switch_page_tables() {
    #[cfg(target_pointer_width = "32")]
    const MODE: satp::Mode = satp::Mode::Sv32;

    #[cfg(target_pointer_width = "64")]
    const MODE: satp::Mode = satp::Mode::Sv39;

    let ppn = kernel_boot_level_0_table.get() >> 12;
    riscv::asm::sfence_vma_all();

    unsafe {
        satp::set(MODE, 0, ppn);
    }

    riscv::asm::fence_i();
}
