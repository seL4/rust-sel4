use crate::sys;

mod arch;
mod fault;
mod object;

pub(crate) mod top_level {
    pub use super::{
        NUM_FAST_MESSAGE_REGISTERS,
        arch::top_level::*,
        fault::{
            CapFault, Fault, NullFault, UnknownSyscall, UserException, VCPUFault, VGICMaintenance,
            VMFault, VPPIEvent,
        },
        object::{ObjectBlueprintArch, ObjectBlueprintArm, ObjectTypeArch, ObjectTypeArm},
    };
}

pub const NUM_FAST_MESSAGE_REGISTERS: usize = sys::seL4_FastMessageRegisters as usize; // no other const way to convert
