use core::arch::riscv64::{fence_i, sfence_vma_all};
use riscv::register::satp;

use crate::translation_tables::kernel::kernel_boot_level_0_table;

pub(crate) fn init_platform_state_per_core(_core_id: usize) {}

pub(crate) fn init_platform_state_per_core_after_which_no_syncronization(_core_id: usize) {
    unsafe {
        kernel_boot_level_0_table.finish();
    }

    unsafe {
        let vaddr = kernel_boot_level_0_table.root() as usize;
        let ppn = vaddr >> 12;

        sfence_vma_all();
        satp::set(satp::Mode::Sv39, 0, ppn);
        fence_i();
    }
}
