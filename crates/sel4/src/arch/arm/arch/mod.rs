use sel4_config::sel4_cfg_if;

sel4_cfg_if! {
    if #[cfg(ARCH_AARCH64)] {
        #[path = "aarch64/mod.rs"]
        mod imp;
    }
}

pub(crate) use imp::*;
