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
        object::{ObjectBlueprintArch, ObjectBlueprintRISCV, ObjectTypeArch, ObjectTypeRISCV},
        user_context::UserContext,
        vm_attributes::VMAttributes,
        vspace::FrameSize,
        NUM_FAST_MESSAGE_REGISTERS,
    };
}

pub const NUM_FAST_MESSAGE_REGISTERS: usize = 4;

pub(crate) mod cap_type_arch {
    use crate::declare_cap_type;

    declare_cap_type!(_4KPage);
    declare_cap_type!(MegaPage);

    #[sel4_config::sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
    declare_cap_type!(GigaPage);

    declare_cap_type!(PageTable);

    pub type VSpace = PageTable;
    pub type Granule = _4KPage;
}

pub(crate) mod local_cptr_arch {
    use crate::declare_local_cptr_alias;

    declare_local_cptr_alias!(_4KPage);
    declare_local_cptr_alias!(MegaPage);

    #[sel4_config::sel4_cfg(any(PT_LEVELS = "3", PT_LEVELS = "4"))]
    declare_local_cptr_alias!(GigaPage);

    declare_local_cptr_alias!(PageTable);
}
