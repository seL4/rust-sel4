//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::asm;

use riscv::register::satp;

use crate::{arch::Arch, main, secondary_main, this_image::kernel_boot_level_0_table, enter_kernel::KernelEntryExtraArgs};

#[unsafe(no_mangle)]
extern "C" fn arch_main(hart_id: usize) -> ! {
    main(KernelEntryExtraArgs { hart_id })
}

#[unsafe(no_mangle)]
extern "C" fn arch_secondary_main(hart_id: usize) -> ! {
    secondary_main(KernelEntryExtraArgs { hart_id })
}

pub(crate) enum ArchImpl {}

impl Arch for ArchImpl {
    fn idle() -> ! {
        loop {
            unsafe {
                asm!("wfi");
            }
        }
    }

    #[allow(unused_variables)]
    fn prepare_to_enter_kernel(core_id: usize) {
        switch_page_tables();
    }
}

fn switch_page_tables() {
    #[cfg(target_pointer_width = "32")]
    const MODE: satp::Mode = satp::Mode::Sv32;

    #[cfg(target_pointer_width = "64")]
    const MODE: satp::Mode = satp::Mode::Sv39;

    unsafe {
        let ppn = kernel_boot_level_0_table.get() >> 12;
        asm!("sfence.vma", options(nostack));
        satp::set(MODE, 0, ppn);
        asm!("fence.i", options(nostack));
    }
}
