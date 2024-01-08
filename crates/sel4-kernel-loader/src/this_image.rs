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
    #[sel4_config::sel4_cfg(ARCH_AARCH64)]
    pub(crate) mod loader {
        include!(concat!(env!("OUT_DIR"), "/loader_page_tables.rs"));
    }
    pub(crate) mod kernel {
        include!(concat!(env!("OUT_DIR"), "/kernel_page_tables.rs"));
    }
}

pub(crate) mod stacks {
    use core::cell::UnsafeCell;

    use sel4_config::sel4_cfg_usize;

    #[repr(C, align(16))]
    struct Stack<const N: usize>(UnsafeCell<[u8; N]>);

    unsafe impl<const N: usize> Sync for Stack<N> {}

    impl<const N: usize> Stack<N> {
        pub const fn new() -> Self {
            Self(UnsafeCell::new([0; N]))
        }

        pub const fn top(&self) -> StackTop {
            StackTop(self.0.get().cast::<u8>().wrapping_add(N))
        }
    }

    #[repr(transparent)]
    pub struct StackTop(#[allow(dead_code)] *mut u8);

    unsafe impl Sync for StackTop {}

    const PRIMARY_STACK_SIZE: usize = 4096 * 8; // TODO this is excessive

    static PRIMARY_STACK: Stack<PRIMARY_STACK_SIZE> = Stack::new();

    #[no_mangle]
    static __primary_stack_top: StackTop = PRIMARY_STACK.top();

    const NUM_SECONDARY_CORES: usize = sel4_cfg_usize!(MAX_NUM_NODES) - 1;

    const SECONDARY_STACK_SIZE: usize = 4096 * 2;
    const SECONDARY_STACKS_SIZE: usize = SECONDARY_STACK_SIZE * NUM_SECONDARY_CORES;

    static SECONDARY_STACKS: Stack<SECONDARY_STACKS_SIZE> = Stack::new();

    pub(crate) fn get_secondary_stack_bottom(core_id: usize) -> usize {
        unsafe {
            SECONDARY_STACKS
                .0
                .get()
                .offset((core_id * SECONDARY_STACK_SIZE).try_into().unwrap()) as usize
        }
    }
}
