#![no_std]

mod gen {
    include!(concat!(env!("OUT_DIR"), "/spec.rs"));
}

pub use gen::SPEC;
