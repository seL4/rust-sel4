#![no_std]

use sel4_platform_info_types::PlatformInfo;

pub const PLATFORM_INFO: PlatformInfo = include!(concat!(env!("OUT_DIR"), "/gen.rs"));
