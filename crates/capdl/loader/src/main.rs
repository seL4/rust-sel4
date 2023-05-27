#![no_std]
#![no_main]
#![feature(const_trait_impl)]
#![feature(int_roundings)]
#![feature(pointer_byte_offsets)]
#![feature(strict_provenance)]

extern crate alloc;

use alloc::vec;
use core::ops::Range;
use core::ptr;
use core::slice;

use capdl_loader_core::{Loader, LoaderBuffers, PerObjectBuffer};
use capdl_loader_types::SpecWithSourcesForSerialization;
use sel4::BootInfo;
use sel4_logging::{LevelFilter, Logger, LoggerBuilder};
use sel4_root_task::main;

const LOG_LEVEL: LevelFilter = LevelFilter::Info;

static LOGGER: Logger = LoggerBuilder::const_default()
    .level_filter(LOG_LEVEL)
    .filter(|meta| meta.target() == "capdl_loader_core")
    .write(|s| sel4::debug_print!("{}", s))
    .build();

#[main]
fn main(bootinfo: &BootInfo) -> ! {
    LOGGER.set().unwrap();
    let spec_with_sources = get_spec_with_sources();
    let mut buffers = LoaderBuffers::new(vec![
        PerObjectBuffer::const_default();
        spec_with_sources.spec.objects.len()
    ]);
    Loader::load(
        bootinfo,
        user_image_bounds(),
        &spec_with_sources,
        &mut buffers,
    )
    .unwrap_or_else(|err| panic!("Error: {}", err))
}

#[no_mangle]
#[link_section = ".data"]
static mut capdl_loader_serialized_spec_start: *mut u8 = ptr::null_mut();

#[no_mangle]
#[link_section = ".data"]
static mut capdl_loader_serialized_spec_size: usize = 0;

#[no_mangle]
#[link_section = ".data"]
static mut capdl_loader_heap_start: *mut u8 = ptr::null_mut();

#[no_mangle]
#[link_section = ".data"]
static mut capdl_loader_heap_size: usize = 0;

#[no_mangle]
#[link_section = ".data"]
static mut capdl_loader_image_start: *mut u8 = ptr::null_mut();

#[no_mangle]
#[link_section = ".data"]
static mut capdl_loader_image_end: *mut u8 = ptr::null_mut();

fn get_spec_with_sources<'a>() -> SpecWithSourcesForSerialization<'a> {
    let blob = unsafe {
        slice::from_raw_parts(
            capdl_loader_serialized_spec_start,
            capdl_loader_serialized_spec_size,
        )
    };
    let (spec, source) = postcard::take_from_bytes(blob).unwrap();
    SpecWithSourcesForSerialization {
        spec,
        object_name_source: source,
        content_source: source,
    }
}

fn user_image_bounds() -> Range<usize> {
    unsafe { capdl_loader_image_start.expose_addr()..capdl_loader_image_end.expose_addr() }
}

fn static_heap_bounds() -> Range<*mut u8> {
    unsafe {
        capdl_loader_heap_start
            ..capdl_loader_heap_start.byte_offset(capdl_loader_heap_size.try_into().unwrap())
    }
}

mod heap {
    use core::ops::Range;

    use sel4_dlmalloc::StaticDlmallocGlobalAlloc;
    use sel4_sync::PanickingMutexSyncOps;

    use super::static_heap_bounds;

    #[global_allocator]
    static GLOBAL_ALLOCATOR: StaticDlmallocGlobalAlloc<
        PanickingMutexSyncOps,
        fn() -> Range<*mut u8>,
    > = StaticDlmallocGlobalAlloc::new(PanickingMutexSyncOps::new(), static_heap_bounds);
}
