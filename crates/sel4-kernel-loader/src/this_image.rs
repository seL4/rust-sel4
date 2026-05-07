//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ops::Range;

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

pub(crate) mod page_tables {
    #[sel4_config::sel4_cfg(ARCH_ARM)]
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

    #[unsafe(no_mangle)]
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
