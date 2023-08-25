#![no_std]
#![allow(clippy::type_complexity)]

mod gen {
    include!(concat!(env!("OUT_DIR"), "/spec.rs"));
}

pub use gen::SPEC;
