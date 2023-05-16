use core::mem;
use core::ops::Neg;

use sel4_loader_payload_types::PayloadInfo;

type KernelEntry = extern "C" fn(
    ui_p_reg_start: usize,
    ui_p_reg_end: usize,
    pv_offset: isize,
    v_entry: usize,
    dtb_addr_p: usize,
    dtb_size: usize,
) -> !;

pub(crate) fn enter_kernel(payload_info: &PayloadInfo) -> ! {
    let kernel_entry = unsafe {
        mem::transmute::<usize, KernelEntry>(
            payload_info.kernel_image.virt_entry.try_into().unwrap(),
        )
    };

    let (dtb_addr_p, dtb_size) = match &payload_info.fdt_phys_addr_range {
        Some(region) => (
            usize::try_from(region.start).unwrap(),
            usize::try_from(region.end).unwrap() - usize::try_from(region.start).unwrap(),
        ),
        None => (0, 0),
    };

    (kernel_entry)(
        payload_info
            .user_image
            .phys_addr_range
            .start
            .try_into()
            .unwrap(),
        payload_info
            .user_image
            .phys_addr_range
            .end
            .try_into()
            .unwrap(),
        payload_info
            .user_image
            .phys_to_virt_offset
            .neg()
            .try_into()
            .unwrap(),
        payload_info.user_image.virt_entry.try_into().unwrap(),
        dtb_addr_p,
        dtb_size,
    )
}
