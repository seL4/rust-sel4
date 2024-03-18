//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use core::ops::Range;
use core::ptr;

use sel4_capdl_initializer_core::{Initializer, InitializerBuffers, PerObjectBuffer};
use sel4_capdl_initializer_types::SpecWithSources;
use sel4_capdl_initializer_with_embedded_spec_embedded_spec::SPEC;
use sel4_logging::{LevelFilter, Logger, LoggerBuilder};

const LOG_LEVEL: LevelFilter =
    // LevelFilter::Trace
    // LevelFilter::Debug
    LevelFilter::Info;

static LOGGER: Logger = LoggerBuilder::const_default()
    .level_filter(LOG_LEVEL)
    .write(|s| sel4::debug_print!("{}", s))
    .build();

static mut BUFFERS: InitializerBuffers<[PerObjectBuffer; SPEC.objects.const_inner().len()]> =
    InitializerBuffers::new([PerObjectBuffer::const_default(); SPEC.objects.const_inner().len()]);

#[sel4_root_task::root_task]
#[allow(clippy::let_unit_value)]
fn main(bootinfo: &sel4::BootInfoPtr) -> ! {
    LOGGER.set().unwrap();
    let trivial_source = ();
    let spec_with_sources = SpecWithSources {
        spec: SPEC,
        object_name_source: &trivial_source,
        content_source: &trivial_source,
        embedded_frame_source: &trivial_source,
    };
    Initializer::initialize(bootinfo, user_image_bounds(), &spec_with_sources, unsafe {
        ptr::addr_of_mut!(BUFFERS).as_mut().unwrap()
    })
}

extern "C" {
    static __executable_start: usize;
    static _end: usize;
}

fn user_image_bounds() -> Range<usize> {
    unsafe { addr_of_ref(&__executable_start)..addr_of_ref(&_end) }
}

fn addr_of_ref<T>(x: &T) -> usize {
    x as *const T as usize
}
