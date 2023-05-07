use sel4_config::sel4_cfg_if;

sel4_cfg_if! {
    if #[cfg(PLAT_QEMU_ARM_VIRT)] {
        #[path = "qemu_arm_virt/mod.rs"]
        mod imp;
    } else if #[cfg(PLAT_BCM2711)] {
        #[path = "bcm2711/mod.rs"]
        mod imp;
    } else {
        compile_error!("Unsupported platform");
    }
}

pub(crate) use imp::*;
