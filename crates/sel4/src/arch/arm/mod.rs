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

pub(crate) mod cap_type_arch {
    use crate::{declare_cap_type, sel4_cfg};

    #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
    declare_cap_type!(VCPU);

    declare_cap_type!(SmallPage);
    declare_cap_type!(LargePage);
    declare_cap_type!(HugePage);
    declare_cap_type!(PGD);
    declare_cap_type!(PUD);
    declare_cap_type!(PD);
    declare_cap_type!(PT);

    pub type VSpace = PGD;
    pub type Granule = SmallPage;
}

pub(crate) mod local_cptr_arch {
    use crate::{declare_local_cptr_alias, sel4_cfg};

    #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
    declare_local_cptr_alias!(VCPU);

    declare_local_cptr_alias!(SmallPage);
    declare_local_cptr_alias!(LargePage);
    declare_local_cptr_alias!(HugePage);
    declare_local_cptr_alias!(PGD);
    declare_local_cptr_alias!(PUD);
    declare_local_cptr_alias!(PD);
    declare_local_cptr_alias!(PT);
}
