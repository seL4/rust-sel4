use sel4_config::sel4_cfg;

#[sel4_cfg(ARCH_X86_64)]
#[path = "x64/mod.rs"]
mod imp;

pub(crate) use imp::*;
