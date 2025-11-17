//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::borrow::BorrowMut;
use core::ops::Range;
use core::ptr;
use core::slice;

#[cfg(feature = "alloc")]
use alloc::vec;

use rkyv::Archive;

use sel4_capdl_initializer_types::SpecForInitializer;
use sel4_immutable_cell::ImmutableCell;
use sel4_logging::{LevelFilter, Logger, LoggerBuilder};
use sel4_root_task::{debug_print, root_task};

use crate::buffers::{InitializerBuffers, PerObjectBuffer};
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

const LOG_LEVEL: LevelFilter = {
    // LevelFilter::Trace
    // LevelFilter::Debug
    LevelFilter::Info
};

static LOGGER: Logger = LoggerBuilder::const_default()
    .level_filter(LOG_LEVEL)
    .filter(|meta| meta.target().starts_with("sel4_capdl_initializer"))
    .write(|s| debug_print!("{}", s))
    .build();

#[cfg_attr(
    feature = "alloc",
    root_task(stack_size = 0x10_000, heap_size = 0x10_000)
)]
#[cfg_attr(not(feature = "alloc"), root_task(stack_size = 0x10_000))]
fn main(bootinfo: &sel4::BootInfoPtr) -> ! {
    LOGGER.set().unwrap();
    let spec = access_spec(get_spec_bytes());
    let mut buffers = InitializerBuffers::new(get_buffers(spec.objects.len()));
    Initializer::initialize(
        bootinfo,
        user_image_bounds(),
        spec,
        *sel4_capdl_initializer_embedded_frames_data_start.get() as usize,
        &mut buffers,
    )
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
fn get_buffers(num_objects: usize) -> impl BorrowMut<[PerObjectBuffer]> {
    vec![PerObjectBuffer::const_default(); num_objects]
}

#[cfg(not(feature = "alloc"))]
fn get_buffers(num_objects: usize) -> impl BorrowMut<[PerObjectBuffer]> {
    use sel4_one_ref_cell::OneRefCell;

    // TODO make configurable
    const MAX_OBJECTS: usize = 4096;

    static BUFFERS: OneRefCell<[PerObjectBuffer; MAX_OBJECTS]> =
        OneRefCell::new([PerObjectBuffer::const_default(); _]);

    assert!(num_objects <= MAX_OBJECTS);

    &mut BUFFERS.take().unwrap().as_mut_slice()[..num_objects]
}

#[cfg(feature = "alloc")]
fn access_spec(bytes: &[u8]) -> &<SpecForInitializer as Archive>::Archived {
    SpecForInitializer::access(bytes).unwrap()
}

#[cfg(not(feature = "alloc"))]
fn access_spec(bytes: &[u8]) -> &<SpecForInitializer as Archive>::Archived {
    unsafe { SpecForInitializer::access_unchecked(bytes) }
}
