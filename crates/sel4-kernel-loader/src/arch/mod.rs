use sel4_config::sel4_cfg;

#[sel4_cfg(ARCH_AARCH64)]
#[path = "aarch64/mod.rs"]
mod imp;

#[sel4_cfg(ARCH_RISCV64)]
#[path = "riscv64/mod.rs"]
mod imp;

pub(crate) use imp::*;
