use crate::sys;

mod arch;
mod object;

pub(crate) mod fault;

pub(crate) mod top_level {
    pub use super::{
        arch::top_level::*,
        fault::Fault,
        object::{ObjectBlueprintArch, ObjectBlueprintX86, ObjectTypeArch, ObjectTypeX86},
        NUM_FAST_MESSAGE_REGISTERS,
    };
}

pub const NUM_FAST_MESSAGE_REGISTERS: usize = sys::seL4_FastMessageRegisters as usize; // no other const way to convert

pub(crate) mod cap_type_arch {
    use crate::declare_cap_type;

    declare_cap_type!(_4K);
    declare_cap_type!(PML4);

    pub type VSpace = PML4;
    pub type Granule = _4K;
}

pub(crate) mod local_cptr_arch {
    use crate::declare_local_cptr_alias;

    declare_local_cptr_alias!(_4K);
    declare_local_cptr_alias!(PML4);
}
