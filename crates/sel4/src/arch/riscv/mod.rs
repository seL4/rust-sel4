mod arch;
mod fault;
mod object;

pub(crate) mod top_level {
    pub use super::{
        NUM_FAST_MESSAGE_REGISTERS,
        arch::top_level::*,
        fault::Fault,
        object::{ObjectBlueprintArch, ObjectBlueprintRISCV, ObjectTypeArch, ObjectTypeRISCV},
    };
}

pub const NUM_FAST_MESSAGE_REGISTERS: usize = 4;
