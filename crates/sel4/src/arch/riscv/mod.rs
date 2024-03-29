//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

mod invocations;
mod object;
mod user_context;
mod vm_attributes;
mod vspace;

pub(crate) mod fault;

pub(crate) mod top_level {
    pub use super::{
        object::{ObjectBlueprintArch, ObjectBlueprintRiscV, ObjectTypeArch, ObjectTypeRISCV},
        user_context::UserContext,
        vm_attributes::VmAttributes,
        vspace::{FrameObjectType, TranslationTableObjectType},
        NUM_FAST_MESSAGE_REGISTERS,
    };
}

pub(crate) use vspace::vspace_levels;

pub const NUM_FAST_MESSAGE_REGISTERS: usize = 4;

pub(crate) mod cap_type_arch {
    use crate::declare_cap_type_for_object_of_fixed_size;

    declare_cap_type_for_object_of_fixed_size!(_4kPage {
        ObjectTypeArch,
        ObjectBlueprintArch
    });
    declare_cap_type_for_object_of_fixed_size!(MegaPage {
        ObjectTypeArch,
        ObjectBlueprintArch
    });

    #[sel4_config::sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
    declare_cap_type_for_object_of_fixed_size!(GigaPage {
        ObjectTypeArch,
        ObjectBlueprintArch
    });

    declare_cap_type_for_object_of_fixed_size!(PageTable {
        ObjectTypeArch,
        ObjectBlueprintArch
    });

    pub type VSpace = PageTable;
    pub type Granule = _4kPage;
}

pub(crate) mod cap_arch {
    use crate::declare_cap_alias;

    declare_cap_alias!(_4kPage);
    declare_cap_alias!(MegaPage);

    #[sel4_config::sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
    declare_cap_alias!(GigaPage);

    declare_cap_alias!(PageTable);
}
