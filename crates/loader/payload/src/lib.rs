#![no_std]

use heapless::Vec;

use loader_payload_types::{DirectRegionContent, ImageInfo, Payload, PayloadInfo, Region};

include!(concat!(env!("OUT_DIR"), "/gen.rs"));
