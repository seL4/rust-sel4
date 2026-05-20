//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use riscv::register::satp;

use sel4_config::{sel4_cfg_if, sel4_cfg_usize};

use crate::{arch::Arch, main, secondary_main, this_image::kernel_boot_level_0_table};

#[unsafe(no_mangle)]
extern "C" fn arch_main(hart_id: usize, dtb: usize) -> ! {
    main(hart_id, dtb)
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
    const MODE: satp::Mode = sel4_cfg_if! {
        if #[sel4_cfg(all(SEL4_ARCH = "riscv64", PT_LEVELS = "3"))] {
            satp::Mode::Sv39
        } else if #[sel4_cfg(all(SEL4_ARCH = "riscv32", PT_LEVELS = "2"))] {
            satp::Mode::Sv32
        } else {
            compiler_error!("unsupported configuration");
        }
    };

    let ppn = kernel_boot_level_0_table.get() >> 12;
    riscv::asm::sfence_vma_all();

    unsafe {
        satp::set(MODE, 0, ppn);
    }

    riscv::asm::fence_i();
}
