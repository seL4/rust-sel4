use core::borrow::Borrow;
use core::ptr;
use core::slice;

use loader_payload_types::Regions;

pub fn copy_payload_data<T: Regions>(regions: &T) {
    for region in regions.iter_regions() {
        let region = region.borrow();
        let dst = unsafe {
            slice::from_raw_parts_mut(
                ptr::from_exposed_addr_mut(region.phys_addr_range.start.try_into().unwrap()),
                (region.phys_addr_range.end - region.phys_addr_range.start)
                    .try_into()
                    .unwrap(),
            )
        };
        match &region.content {
            Some(src) => {
                dst.copy_from_slice(src.borrow());
            }
            None => {
                // NOTE slice::fill is too slow
                // dst.fill(0);
                unsafe {
                    ptr::write_bytes(dst.as_mut_ptr(), 0, dst.len());
                }
            }
        }
    }
}
