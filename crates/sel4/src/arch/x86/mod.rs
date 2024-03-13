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

    declare_cap_type!(IOPortControl);
}

pub(crate) mod cap_arch {
    use crate::declare_cap_alias;

    declare_cap_alias!(_4K);
    declare_cap_alias!(LargePage);
    declare_cap_alias!(HugePage);

    declare_cap_alias!(PML4);
    declare_cap_alias!(PDPT);
    declare_cap_alias!(PageDirectory);
    declare_cap_alias!(PageTable);

    declare_cap_alias!(IOPortControl);
}
