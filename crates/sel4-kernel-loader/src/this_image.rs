use core::ops::Range;
use core::slice;

use sel4_immutable_cell::ImmutableCell;
use sel4_kernel_loader_payload_types::*;

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

pub(crate) fn get_payload() -> (Payload<usize>, &'static [u8]) {
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

pub(crate) mod page_tables {
    #[sel4_config::sel4_cfg(ARCH_AARCH64)]
    pub(crate) mod loader {
        include!(concat!(env!("OUT_DIR"), "/loader_page_tables.rs"));
    }
    pub(crate) mod kernel {
        include!(concat!(env!("OUT_DIR"), "/kernel_page_tables.rs"));
    }
}

pub(crate) mod stacks {
    use core::sync::Exclusive;

    use sel4_config::sel4_cfg_usize;

    #[repr(C, align(16))]
    struct Stack<const N: usize>([u8; N]);

    const PRIMARY_STACK_SIZE: usize = 4096 * 8; // TODO this is excessive

    static mut PRIMARY_STACK: Stack<PRIMARY_STACK_SIZE> = Stack([0; PRIMARY_STACK_SIZE]);

    #[no_mangle]
    static __primary_stack_bottom: Exclusive<*const u8> =
        Exclusive::new(unsafe { PRIMARY_STACK.0.as_ptr_range().end });

    const NUM_SECONDARY_CORES: usize = sel4_cfg_usize!(MAX_NUM_NODES) - 1;

    const SECONDARY_STACK_SIZE: usize = 4096 * 2;
    const SECONDARY_STACKS_SIZE: usize = SECONDARY_STACK_SIZE * NUM_SECONDARY_CORES;

    static SECONDARY_STACKS: Stack<SECONDARY_STACKS_SIZE> = Stack([0; SECONDARY_STACKS_SIZE]);

    pub(crate) fn get_secondary_stack_bottom(core_id: usize) -> usize {
        unsafe {
            SECONDARY_STACKS
                .0
                .as_ptr()
                .offset((core_id * SECONDARY_STACK_SIZE).try_into().unwrap())
                .expose_addr()
        }
    }
}
