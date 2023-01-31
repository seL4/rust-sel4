#![no_std]
#![no_main]
#![feature(const_trait_impl)]

use capdl_embedded_spec::SPEC;
use capdl_loader_core::{load, LoaderBuffers};
use sel4::BootInfo;
use sel4_logging::{LevelFilter, Logger, LoggerBuilder};

const LOG_LEVEL: LevelFilter = LevelFilter::Info;

static LOGGER: Logger = LoggerBuilder::default()
    .level_filter(LOG_LEVEL)
    .write(|s| sel4::debug_print!("{}", s))
    .build();

static mut BUFFERS: LoaderBuffers<{ SPEC.objects.as_slice().len() }> = LoaderBuffers::new();

#[sel4_minimal_root_task_runtime::main]
fn main(bootinfo: &BootInfo) -> ! {
    LOGGER.set().unwrap();
    load(&SPEC, &bootinfo, unsafe { &mut BUFFERS }).unwrap_or_else(|err| panic!("Error: {}", err))
}
