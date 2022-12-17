use sel4_config::sel4_cfg_if;

// [TODO]
// sel4-config doesn't yet play nicely with:
//   - ARCH_ARM
//   - ARCH_RISCV
//   - ARCH_X86

sel4_cfg_if! {
    if #[cfg(ARCH_AARCH64)] {
        #[path = "arm/mod.rs"]
        mod imp;
    } else if #[cfg(ARCH_X86_64)] {
        #[path = "x86/mod.rs"]
        mod imp;
    }
}

pub(crate) use imp::*;
