use sel4_config::sel4_cfg;

#[sel4_cfg(ARCH_AARCH64)]
#[path = "aarch64/mod.rs"]
mod imp;

#[sel4_cfg(any(ARCH_RISCV64, ARCH_RISCV32))]
#[path = "riscv/mod.rs"]
mod imp;

pub(crate) use imp::*;
