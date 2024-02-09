//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use crate::sys;

mod arch;
mod object;
mod vm_attributes;

pub(crate) mod fault;

pub(crate) mod top_level {
    pub use super::{
        arch::top_level::*,
        object::{ObjectBlueprintArch, ObjectBlueprintX86, ObjectTypeArch, ObjectTypeX86},
        vm_attributes::VmAttributes,
        NUM_FAST_MESSAGE_REGISTERS,
    };
}

pub const NUM_FAST_MESSAGE_REGISTERS: usize = sys::seL4_FastMessageRegisters as usize; // no other const way to convert

pub(crate) mod cap_type_arch {
    use crate::declare_cap_type;

    declare_cap_type!(_4K);
    declare_cap_type!(LargePage);
    declare_cap_type!(HugePage);

    declare_cap_type!(PML4);
    declare_cap_type!(PDPT);
    declare_cap_type!(PageDirectory);
    declare_cap_type!(PageTable);

    pub type VSpace = PML4;
    pub type Granule = _4K;
}

pub(crate) mod local_cptr_arch {
    use crate::declare_local_cptr_alias;

    declare_local_cptr_alias!(_4K);
    declare_local_cptr_alias!(LargePage);
    declare_local_cptr_alias!(HugePage);

    declare_local_cptr_alias!(PML4);
    declare_local_cptr_alias!(PDPT);
    declare_local_cptr_alias!(PageDirectory);
    declare_local_cptr_alias!(PageTable);
}
