use core::arch::asm;
use core::mem;

use riscv::register::satp;

use sel4_kernel_loader_payload_types::PayloadInfo;

use crate::{
    arch::Arch, main, secondary_main, this_image::page_tables::kernel::kernel_boot_level_0_table,
};

#[no_mangle]
static mut hsm_exists: i32 = 0;

#[no_mangle]
extern "C" fn arch_main() -> ! {
    main(())
}

#[no_mangle]
extern "C" fn arch_secondary_main() -> ! {
    secondary_main(())
}

pub(crate) enum ArchImpl {}

impl Arch for ArchImpl {
    type PerCore = ();

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

        switch_page_tables();

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
