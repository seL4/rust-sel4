use core::ops::Range;
use core::slice;

use sel4_immutable_cell::ImmutableCell;
use sel4_kernel_loader_payload_types::*;

pub(crate) fn get_payload() -> (ConcretePayload, &'static [u8]) {
    let blob = unsafe {
        slice::from_raw_parts(
            *loader_payload_start.get() as *const u8,
            *loader_payload_size.get(),
        )
    };
    let (payload, source) = postcard::take_from_bytes(blob).unwrap();
    (payload, source)
}

pub(crate) fn get_user_image_bounds() -> Range<usize> {
    *loader_image_start.get()..*loader_image_end.get()
}

#[no_mangle]
#[link_section = ".data"]
static loader_payload_start: ImmutableCell<usize> = ImmutableCell::new(0);

#[no_mangle]
#[link_section = ".data"]
static loader_payload_size: ImmutableCell<usize> = ImmutableCell::new(0);

#[no_mangle]
#[link_section = ".data"]
static loader_image_start: ImmutableCell<usize> = ImmutableCell::new(0);

#[no_mangle]
#[link_section = ".data"]
static loader_image_end: ImmutableCell<usize> = ImmutableCell::new(0);
