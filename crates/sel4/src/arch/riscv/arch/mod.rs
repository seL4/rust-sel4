use sel4_config::sel4_cfg;

#[sel4_cfg(ARCH_RISCV64)]
#[path = "riscv64/mod.rs"]
mod imp;

pub(crate) use imp::*;
