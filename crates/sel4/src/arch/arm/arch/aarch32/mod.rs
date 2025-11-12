//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

mod fault;
mod object;
mod user_context;

sel4_config::sel4_cfg_if! {
    if #[sel4_cfg(ARM_HYPERVISOR_SUPPORT)] {
        mod vcpu_reg;
    }
}

// HACK for rustfmt
#[cfg(false)]
mod vcpu_reg;

pub(crate) mod top_level {
    pub use super::{
        object::{
            ObjectBlueprintAArch32, ObjectBlueprintSeL4Arch, ObjectTypeAArch32, ObjectTypeSeL4Arch,
        },
        user_context::UserContext,
    };

    #[sel4_config::sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
    pub use super::vcpu_reg::VCpuReg;
}
