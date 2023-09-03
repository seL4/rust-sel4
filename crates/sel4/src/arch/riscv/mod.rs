mod arch;
mod object;
mod vm_attributes;

pub(crate) mod fault;

pub(crate) mod top_level {
    pub use super::{
        arch::top_level::*,
        object::{ObjectBlueprintArch, ObjectBlueprintRISCV, ObjectTypeArch, ObjectTypeRISCV},
        vm_attributes::VMAttributes,
        NUM_FAST_MESSAGE_REGISTERS,
    };
}

pub const NUM_FAST_MESSAGE_REGISTERS: usize = 4;

pub(crate) mod cap_type_arch {
    use crate::declare_cap_type;

    declare_cap_type!(_4KPage);
    declare_cap_type!(MegaPage);
    declare_cap_type!(GigaPage);
    declare_cap_type!(PageTable);

    pub type VSpace = PageTable;
    pub type Granule = _4KPage;
}

pub(crate) mod local_cptr_arch {
    use crate::declare_local_cptr_alias;

    declare_local_cptr_alias!(_4KPage);
    declare_local_cptr_alias!(MegaPage);
    declare_local_cptr_alias!(GigaPage);
    declare_local_cptr_alias!(PageTable);
}
