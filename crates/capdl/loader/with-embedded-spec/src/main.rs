#![no_std]
#![no_main]
#![feature(const_trait_impl)]
#![feature(strict_provenance)]

use core::ops::Range;

use capdl_embedded_spec::SPEC;
use capdl_loader_core::{Loader, LoaderBuffers, PerObjectBuffer};
use sel4::BootInfo;
use sel4_logging::{LevelFilter, Logger, LoggerBuilder};

const LOG_LEVEL: LevelFilter = LevelFilter::Info;

static LOGGER: Logger = LoggerBuilder::default()
    .level_filter(LOG_LEVEL)
    .write(|s| sel4::debug_print!("{}", s))
    .build();

static mut BUFFERS: LoaderBuffers<[PerObjectBuffer; SPEC.objects.len()]> =
    LoaderBuffers::new([PerObjectBuffer::default(); SPEC.objects.len()]);

#[sel4_minimal_root_task_runtime::main]
fn main(bootinfo: &BootInfo) -> ! {
    LOGGER.set().unwrap();
    Loader::load(&bootinfo, user_image_bounds(), &SPEC, &(), unsafe {
        &mut BUFFERS
    })
    .unwrap_or_else(|err| panic!("Error: {}", err))
}

extern "C" {
    static __executable_start: u64;
    static _end: u64;
}

fn user_image_bounds() -> Range<usize> {
    unsafe { addr_of_ref(&__executable_start)..addr_of_ref(&_end) }
}

fn addr_of_ref<T>(x: &T) -> usize {
    (x as *const T).addr()
}
