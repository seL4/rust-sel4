use core::fmt;

pub(crate) mod types;

use types::Bitfield;

include!(concat!(env!("OUT_DIR"), "/types.rs"));
include!(concat!(env!("OUT_DIR"), "/shared_types.rs"));
