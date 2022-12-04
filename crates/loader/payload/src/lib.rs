#![no_std]

use loader_payload_types::{ImageInfo, Payload, PayloadInfo, Region};

include!(concat!(env!("OUT_DIR"), "/gen.rs"));
