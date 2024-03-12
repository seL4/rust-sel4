//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use crate::{const_helpers::u32_into_usize, sys};

mod arch;
mod invocations;
mod object;
mod vm_attributes;
mod vspace;

pub(crate) mod fault;

pub(crate) mod top_level {
    pub use super::{
        arch::top_level::*,
        object::{ObjectBlueprintArch, ObjectBlueprintX86, ObjectTypeArch, ObjectTypeX86},
        vm_attributes::VmAttributes,
        vspace::{FrameObjectType, TranslationStructureObjectType},
        NUM_FAST_MESSAGE_REGISTERS,
    };
}

pub(crate) use vspace::vspace_levels;

pub const NUM_FAST_MESSAGE_REGISTERS: usize = u32_into_usize(sys::seL4_FastMessageRegisters);

pub(crate) mod cap_type_arch {
    use crate::{declare_cap_type, declare_cap_type_for_object_of_fixed_size};

    declare_cap_type_for_object_of_fixed_size!(_4k {
        ObjectTypeArch,
        ObjectBlueprintArch
    });
    declare_cap_type_for_object_of_fixed_size!(LargePage {
        ObjectTypeArch,
        ObjectBlueprintArch
    });
    declare_cap_type_for_object_of_fixed_size!(HugePage {
        ObjectTypeSeL4Arch,
        ObjectBlueprintSeL4Arch
    });

    declare_cap_type_for_object_of_fixed_size!(PML4 {
        ObjectTypeSeL4Arch,
        ObjectBlueprintSeL4Arch
    });
    declare_cap_type_for_object_of_fixed_size!(PDPT {
        ObjectTypeSeL4Arch,
        ObjectBlueprintSeL4Arch
    });
    declare_cap_type_for_object_of_fixed_size!(PageDirectory {
        ObjectTypeArch,
        ObjectBlueprintArch
    });
    declare_cap_type_for_object_of_fixed_size!(PageTable {
        ObjectTypeArch,
        ObjectBlueprintArch
    });

    pub type VSpace = PML4;
    pub type Granule = _4k;

    declare_cap_type!(IOPortControl);
}

pub(crate) mod cap_arch {
    use crate::declare_cap_alias;

    declare_cap_alias!(_4k);
    declare_cap_alias!(LargePage);
    declare_cap_alias!(HugePage);

    declare_cap_alias!(PML4);
    declare_cap_alias!(PDPT);
    declare_cap_alias!(PageDirectory);
    declare_cap_alias!(PageTable);

    declare_cap_alias!(IOPortControl);
}
