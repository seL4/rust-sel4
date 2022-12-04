use crate::bf::*;
use crate::c::*;

pub mod invocation_label {
    include!(concat!(env!("OUT_DIR"), "/invocation_labels.rs"));
}

include!(concat!(env!("OUT_DIR"), "/invocations.rs"));
