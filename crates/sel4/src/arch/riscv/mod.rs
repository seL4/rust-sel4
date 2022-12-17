mod arch;
mod fault;
mod object;

pub(crate) mod top_level {
    pub use super::{
        arch::top_level::*,
        fault::Fault,
        object::{ObjectBlueprintArch, ObjectBlueprintRISCV, ObjectTypeArch, ObjectTypeRISCV},
        NUM_FAST_MESSAGE_REGISTERS,
    };
}

pub const NUM_FAST_MESSAGE_REGISTERS: usize = 4;
