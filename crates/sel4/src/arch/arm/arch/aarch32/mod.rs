mod fault;
mod object;
mod user_context;

#[sel4_config::sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
mod vcpu_reg;

pub(crate) mod top_level {
    pub use super::{
        object::{
            ObjectBlueprintAArch64, ObjectBlueprintSeL4Arch, ObjectTypeAArch64, ObjectTypeSeL4Arch,
        },
        user_context::UserContext,
    };

    #[sel4_config::sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
    pub use super::vcpu_reg::VCPUReg;
}
