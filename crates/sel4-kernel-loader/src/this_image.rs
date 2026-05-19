//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ops::Range;

use sel4_immutable_cell::ImmutableCell;
use sel4_kernel_loader_payload_types::*;
use sel4_phdrs::{PT_SEL4_KERNEL_LOADER_PAYLOAD, locate_phdrs};

use sel4_phdrs_patched as _;

pub(crate) fn get_payload() -> &'static ArchivedPayload {
    unsafe {
        Payload::access_unchecked(
            locate_phdrs()
                .unwrap()
                .find_by_type(PT_SEL4_KERNEL_LOADER_PAYLOAD)
                .unwrap()
                .bytes(),
        )
    }
}

pub(crate) fn get_user_image_bounds() -> Range<usize> {
    locate_phdrs().unwrap().footprint().unwrap()
}

#[sel4_config::sel4_cfg(ARCH_ARM)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".data")]
static loader_level_0_table: ImmutableCell<usize> = ImmutableCell::new(0);

#[unsafe(no_mangle)]
#[unsafe(link_section = ".data")]
pub(crate) static kernel_boot_level_0_table: ImmutableCell<usize> = ImmutableCell::new(0);

pub(crate) mod stacks {
    use sel4_config::sel4_cfg_usize;
    use sel4_stack::{Stack, StackBottom};

    const PRIMARY_STACK_SIZE: usize = 4096 * 8; // TODO this is excessive

    static PRIMARY_STACK: Stack<PRIMARY_STACK_SIZE> = Stack::new();

    #[unsafe(no_mangle)]
    static __primary_stack_bottom: StackBottom = PRIMARY_STACK.bottom();

    const MAX_NUM_NODES: usize = sel4_cfg_usize!(MAX_NUM_NODES);

    const SECONDARY_STACK_SIZE: usize = 4096 * 2;

    static SECONDARY_STACKS: [Stack<SECONDARY_STACK_SIZE>; MAX_NUM_NODES] =
        [const { Stack::new() }; MAX_NUM_NODES];

    pub(crate) fn get_secondary_stack_bottom(core_id: usize) -> StackBottom {
        SECONDARY_STACKS[core_id].bottom()
    }
}
