use core::arch::asm;

use riscv::register::satp;

use crate::this_image::page_tables::kernel::kernel_boot_level_0_table;

#[no_mangle]
static mut hsm_exists: i32 = 0;

pub(crate) fn idle() -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

pub(crate) fn init_platform_state_per_core(_core_id: usize) {}

pub(crate) fn init_platform_state_per_core_after_which_no_syncronization(_core_id: usize) {
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
        kernel_boot_level_0_table.finish();
        let ppn = kernel_boot_level_0_table.root() as usize >> 12;
        by_ptr_width(ppn);
    }
}
