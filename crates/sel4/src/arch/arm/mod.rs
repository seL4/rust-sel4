//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use crate::sys;

mod arch;
mod invocations;
mod object;
mod vm_attributes;
mod vspace;

pub(crate) mod fault;

pub(crate) mod top_level {
    pub use super::{
        arch::top_level::*,
        object::{ObjectBlueprintArch, ObjectBlueprintArm, ObjectTypeArch, ObjectTypeArm},
        vm_attributes::VmAttributes,
        vspace::FrameSize,
        NUM_FAST_MESSAGE_REGISTERS,
    };
}

/// The number of message registers which are passed in architectural registers.
pub const NUM_FAST_MESSAGE_REGISTERS: usize = sys::seL4_FastMessageRegisters as usize; // no other const way to convert

pub(crate) mod cap_type_arch {
    use crate::{declare_cap_type, sel4_cfg};

    #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
    declare_cap_type! {
        /// Corresponds to `seL4_ARM_VCPU`.
        VCpu
    }

    declare_cap_type! {
        /// Corresponds to `seL4_ARM_Page` with `size_bits = 12`.
        SmallPage
    }

    declare_cap_type! {
        /// Corresponds to `seL4_ARM_Page` with `size_bits = 21`.
        LargePage
    }

    #[sel4_cfg(ARCH_AARCH64)]
    declare_cap_type! {
        /// Corresponds to `seL4_ARM_Page` with `size_bits = 30`.
        HugePage
    }

    #[sel4_cfg(ARCH_AARCH64)]
    declare_cap_type! {
        /// Corresponds to `seL4_ARM_VSpace`.
        VSpace
    }

    #[sel4_cfg(ARCH_AARCH32)]
    declare_cap_type! {
        /// Corresponds to `seL4_ARM_PD`.
        PD
    }

    declare_cap_type! {
        /// Corresponds to `seL4_ARM_PageTable`.
        PT
    }

    /// Alias for [`cap_type::SmallPage`](SmallPage).
    pub type Granule = SmallPage;

    #[sel4_cfg(ARCH_AARCH32)]
    /// Alias for [`cap_type::PD`](PD).
    pub type VSpace = PD;
}

pub(crate) mod cap_arch {
    use crate::{declare_cap_alias, sel4_cfg};

    #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
    declare_cap_alias!(VCpu);

    declare_cap_alias!(SmallPage);
    declare_cap_alias!(LargePage);

    #[sel4_cfg(ARCH_AARCH64)]
    declare_cap_alias!(HugePage);

    #[sel4_cfg(ARCH_AARCH32)]
    declare_cap_alias!(PD);

    declare_cap_alias!(PT);
}
