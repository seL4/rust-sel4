use sel4_config::sel4_cfg_if;

sel4_cfg_if! {
    if #[cfg(ARCH_X86_64)] {
        #[path = "x64/mod.rs"]
        mod imp;
    }
}

pub(crate) use imp::*;
