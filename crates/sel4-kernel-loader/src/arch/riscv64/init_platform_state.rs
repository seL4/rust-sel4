use core::arch::riscv64::{fence_i, sfence_vma_all};
use riscv::register::satp;

extern "C" {
    static kernel_boot_level_0_table: u64;
    static kernel_boot_level_0_table_num_total_entries: usize;
}

pub(crate) fn init_platform_state_per_core(_core_id: usize) {}

pub(crate) fn init_platform_state_per_core_after_which_no_syncronization(_core_id: usize) {
    unsafe {
        let ptr = &kernel_boot_level_0_table as *const u64 as *mut u64;
        let vaddr = ptr as usize;
        let ppn = vaddr >> 12;

        let entries =
            core::slice::from_raw_parts_mut(ptr, kernel_boot_level_0_table_num_total_entries);
        for entry in entries.iter_mut() {
            *entry = entry.rotate_right(2);
        }

        sfence_vma_all();
        satp::set(satp::Mode::Sv39, 0, ppn);
        fence_i();
    }
}
