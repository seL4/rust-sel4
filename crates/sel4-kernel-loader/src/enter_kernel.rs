use core::mem;

use sel4_kernel_loader_payload_types::PayloadInfo;

type KernelEntry = extern "C" fn(
    ui_p_reg_start: usize,
    ui_p_reg_end: usize,
    pv_offset: isize,
    v_entry: usize,
    dtb_addr_p: usize,
    dtb_size: usize,
) -> !;

pub(crate) fn enter_kernel(payload_info: &PayloadInfo<usize>) -> ! {
    let kernel_entry =
        unsafe { mem::transmute::<usize, KernelEntry>(payload_info.kernel_image.virt_entry) };

    let (dtb_addr_p, dtb_size) = match &payload_info.fdt_phys_addr_range {
        Some(region) => (
            usize::try_from(region.start).unwrap(),
            usize::try_from(region.end).unwrap() - usize::try_from(region.start).unwrap(),
        ),
        None => (0, 0),
    };

    (kernel_entry)(
        payload_info.user_image.phys_addr_range.start,
        payload_info.user_image.phys_addr_range.end,
        0_usize.wrapping_sub(payload_info.user_image.phys_to_virt_offset) as isize,
        payload_info.user_image.virt_entry,
        dtb_addr_p,
        dtb_size,
    )
}
