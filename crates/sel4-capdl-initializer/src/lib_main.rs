//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ops::Range;
use core::ptr;
use core::slice;

use rkyv::Archive;

use sel4_capdl_initializer_types::SpecForInitializer;
use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;
use sel4_immutable_cell::ImmutableCell;
use sel4_logging::{LevelFilter, Logger, LoggerBuilder};
use sel4_root_task::{debug_print, root_task};

use crate::initialize::Initializer;

#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".data")]
static sel4_capdl_initializer_serialized_spec_data_start: ImmutableCell<*mut u8> =
    ImmutableCell::new(ptr::null_mut());

#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".data")]
static sel4_capdl_initializer_serialized_spec_data_size: ImmutableCell<usize> =
    ImmutableCell::new(0);

#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".data")]
static sel4_capdl_initializer_embedded_frames_data_start: ImmutableCell<*mut u8> =
    ImmutableCell::new(ptr::null_mut());

#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".data")]
static sel4_capdl_initializer_image_start: ImmutableCell<*mut u8> =
    ImmutableCell::new(ptr::null_mut());

#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".data")]
static sel4_capdl_initializer_image_end: ImmutableCell<*mut u8> =
    ImmutableCell::new(ptr::null_mut());

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
        *sel4_capdl_initializer_embedded_frames_data_start.get() as usize,
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
        slice::from_raw_parts(
            *sel4_capdl_initializer_serialized_spec_data_start.get(),
            *sel4_capdl_initializer_serialized_spec_data_size.get(),
        )
    }
}

fn user_image_bounds() -> Range<usize> {
    (*sel4_capdl_initializer_image_start.get() as usize)
        ..(*sel4_capdl_initializer_image_end.get() as usize)
}

#[cfg(feature = "alloc")]
fn access_spec(bytes: &[u8]) -> &<SpecForInitializer as Archive>::Archived {
    SpecForInitializer::access(bytes).unwrap()
}

#[cfg(not(feature = "alloc"))]
fn access_spec(bytes: &[u8]) -> &<SpecForInitializer as Archive>::Archived {
    unsafe { SpecForInitializer::access_unchecked(bytes) }
}
