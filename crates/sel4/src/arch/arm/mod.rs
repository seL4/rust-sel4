use crate::sys;

mod arch;
mod object;

pub(crate) mod fault;

pub(crate) mod top_level {
    pub use super::{
        arch::top_level::*,
        object::{ObjectBlueprintArch, ObjectBlueprintArm, ObjectTypeArch, ObjectTypeArm},
        NUM_FAST_MESSAGE_REGISTERS,
    };
}

pub const NUM_FAST_MESSAGE_REGISTERS: usize = sys::seL4_FastMessageRegisters as usize; // no other const way to convert
