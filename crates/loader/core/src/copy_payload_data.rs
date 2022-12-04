use core::ptr;
use core::slice;

use loader_payload_types::Region;

pub fn copy_payload_data(regions: &[Region<&[u8]>]) {
    for region in regions {
        let dst = unsafe {
            slice::from_raw_parts_mut(
                <*mut u8>::from_bits(region.phys_addr_range.start.try_into().unwrap()),
                (region.phys_addr_range.end - region.phys_addr_range.start)
                    .try_into()
                    .unwrap(),
            )
        };
        match region.content {
            Some(src) => {
                dst.copy_from_slice(src);
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
