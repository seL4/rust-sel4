//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ops::Range;

use rkyv::Archive;

use sel4_capdl_initializer_types::SpecForInitializer;
use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;
use sel4_logging::{LevelFilter, Logger, LoggerBuilder};
use sel4_phdrs::locate_phdrs;
use sel4_root_task::{debug_print, root_task};

use sel4_phdrs_patched as _;

use crate::initialize::Initializer;

const PT_SEL4_CAPDL_SPEC: u32 = 0x64c3_4002;
const PT_SEL4_CAPDL_FRAME_DATA: u32 = 0x64c3_4003;

static LOGGER: ImmediateSyncOnceCell<Logger> = ImmediateSyncOnceCell::new();

#[cfg_attr(
    feature = "alloc",
    root_task(stack_size = 0x10_000, heap_size = 0x10_000)
)]
#[cfg_attr(not(feature = "alloc"), root_task(stack_size = 0x10_000))]
fn main(bootinfo: &sel4::BootInfoPtr) -> ! {
    let spec = access_spec(get_spec_bytes());
    init_logging(spec.log_level.unwrap());
    Initializer::initialize(
        bootinfo,
        user_image_bounds(),
        spec,
        locate_phdrs()
            .unwrap()
            .find_by_type(PT_SEL4_CAPDL_FRAME_DATA)
            .unwrap()
            .p_vaddr,
    )
}

fn init_logging(spec_log_level: u8) {
    let mut level_filter = LevelFilter::Off;
    for _ in 0..spec_log_level {
        level_filter = level_filter.increment_severity();
    }
    LOGGER
        .set(
            LoggerBuilder::default()
                .level_filter(level_filter)
                .write(|s| debug_print!("{}", s))
                .build(),
        )
        .ok()
        .unwrap();
    LOGGER.get().unwrap().set().unwrap();
}

fn get_spec_bytes() -> &'static [u8] {
    unsafe {
        locate_phdrs()
            .unwrap()
            .find_by_type(PT_SEL4_CAPDL_SPEC)
            .unwrap()
            .bytes()
    }
}

fn user_image_bounds() -> Range<usize> {
    locate_phdrs().unwrap().footprint().unwrap()
}

#[cfg(feature = "alloc")]
fn access_spec(bytes: &[u8]) -> &<SpecForInitializer as Archive>::Archived {
    SpecForInitializer::access(bytes).unwrap()
}

#[cfg(not(feature = "alloc"))]
fn access_spec(bytes: &[u8]) -> &<SpecForInitializer as Archive>::Archived {
    unsafe { SpecForInitializer::access_unchecked(bytes) }
}
