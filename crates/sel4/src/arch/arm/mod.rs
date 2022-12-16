mod arch;
mod object;
mod fault;

pub(crate) mod top_level {
    pub use super::{
        object::{ObjectBlueprintArch, ObjectTypeArch, ObjectBlueprintArm, ObjectTypeArm},
        fault::{
            Fault,
            CapFault,
            NullFault,
            UnknownSyscall,
            UserException,
            VCPUFault,
            VGICMaintenance,
            VPPIEvent,
            VMFault,
        },

        arch::top_level::*,
    };
}
