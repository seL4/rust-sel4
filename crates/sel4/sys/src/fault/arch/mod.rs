use sel4_config::sel4_cfg_if;

sel4_cfg_if! {
    if #[cfg(ARCH_AARCH64)] {
        #[path = "aarch64.rs"]
        mod imp;
    } else if #[cfg(ARCH_RISCV64)] {
        #[path = "riscv64.rs"]
        mod imp;
    } else if #[cfg(ARCH_X86_64)] {
        #[path = "x86_64.rs"]
        mod imp;
    }
}

pub use imp::*;
