use crate::sys;

mod arch;
mod fault;
mod object;

pub(crate) mod top_level {
    pub use super::{
        NUM_FAST_MESSAGE_REGISTERS,
        arch::top_level::*,
        fault::Fault,
        object::{ObjectBlueprintArch, ObjectBlueprintX86, ObjectTypeArch, ObjectTypeX86},
    };
}

pub const NUM_FAST_MESSAGE_REGISTERS: usize = sys::seL4_FastMessageRegisters as usize; // no other const way to convert
