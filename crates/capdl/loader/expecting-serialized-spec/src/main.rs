#![no_std]
#![no_main]
#![feature(const_trait_impl)]
#![feature(strict_provenance)]

extern crate alloc;

use alloc::vec;
use core::ops::Range;
use core::ptr;
use core::slice;

use capdl_loader_core::{Loader, LoaderBuffers, PerObjectBuffer};
use capdl_loader_expecting_serialized_spec_types::SpecWithSourcesForSerialization;
use sel4::BootInfo;
use sel4_logging::{LevelFilter, Logger, LoggerBuilder};

const LOG_LEVEL: LevelFilter = LevelFilter::Info;

static LOGGER: Logger = LoggerBuilder::default()
    .level_filter(LOG_LEVEL)
    .write(|s| sel4::debug_print!("{}", s))
    .build();

#[sel4_minimal_root_task_runtime::main]
fn main(bootinfo: &BootInfo) -> ! {
    LOGGER.set().unwrap();
    let spec_with_sources = get_spec_with_sources();
    let mut buffers = LoaderBuffers::new(vec![
        PerObjectBuffer::default();
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

#[used]
#[no_mangle]
#[link_section = ".data"]
static mut capdl_spec_start: *const u8 = ptr::null();

#[used]
#[no_mangle]
#[link_section = ".data"]
static mut capdl_spec_size: usize = 0;

fn get_spec_with_sources<'a>() -> SpecWithSourcesForSerialization<'a> {
    let blob = unsafe { slice::from_raw_parts(capdl_spec_start, capdl_spec_size) };
    let (spec, source) = postcard::take_from_bytes(blob).unwrap();
    SpecWithSourcesForSerialization {
        spec,
        object_name_source: source,
        content_source: source,
    }
}

extern "C" {
    static __executable_start: u64;
}

fn user_image_bounds() -> Range<usize> {
    unsafe { addr_of_ref(&__executable_start)..(capdl_spec_start.addr() + capdl_spec_size) }
}

fn addr_of_ref<T>(x: &T) -> usize {
    (x as *const T).addr()
}
