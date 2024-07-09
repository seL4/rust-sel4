//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

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
    #[sel4_config::sel4_cfg(any(ARCH_AARCH64, ARCH_AARCH32))]
    pub(crate) mod loader {
        include!(concat!(env!("OUT_DIR"), "/loader_page_tables.rs"));
    }
    pub(crate) mod kernel {
        include!(concat!(env!("OUT_DIR"), "/kernel_page_tables.rs"));
    }
}

pub(crate) mod stacks {
    use sel4_config::sel4_cfg_usize;
    use sel4_stack::{Stack, StackBottom};

    const PRIMARY_STACK_SIZE: usize = 4096 * 8; // TODO this is excessive

    static PRIMARY_STACK: Stack<PRIMARY_STACK_SIZE> = Stack::new();

    #[no_mangle]
    static __primary_stack_bottom: StackBottom = PRIMARY_STACK.bottom();

    const NUM_SECONDARY_CORES: usize = sel4_cfg_usize!(MAX_NUM_NODES) - 1;

    const SECONDARY_STACK_SIZE: usize = 4096 * 2;

    static SECONDARY_STACKS: [Stack<SECONDARY_STACK_SIZE>; NUM_SECONDARY_CORES] =
        [const { Stack::new() }; NUM_SECONDARY_CORES];

    #[allow(clippy::zst_offset)] // for case where NUM_SECONDARY_CORES == 0
    pub(crate) fn get_secondary_stack_bottom(core_id: usize) -> StackBottom {
        assert!(core_id > 0 && core_id < sel4_cfg_usize!(MAX_NUM_NODES));
        SECONDARY_STACKS[core_id - 1].bottom()
    }
}
