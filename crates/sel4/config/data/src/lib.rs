#![allow(unused_imports)]

use lazy_static::lazy_static;

pub use sel4_config_generic_types::{Configuration, Key, Value};

pub fn get_kernel_config() -> &'static Configuration {
    &KERNEL_CONFIGURATION
}

lazy_static! {
    static ref KERNEL_CONFIGURATION: Configuration = mk();
}

mod helpers {
    pub(crate) use sel4_config_generic_types::{Configuration, Value};
    pub(crate) use std::string::ToString;
}

include!(concat!(env!("OUT_DIR"), "/gen.rs"));
