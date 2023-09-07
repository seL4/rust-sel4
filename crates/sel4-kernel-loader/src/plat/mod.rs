use sel4_config::sel4_cfg;

#[sel4_cfg(all(ARCH_AARCH64, PLAT_QEMU_ARM_VIRT))]
#[path = "qemu_arm_virt/mod.rs"]
mod imp;

#[sel4_cfg(all(ARCH_AARCH64, PLAT_BCM2711))]
#[path = "bcm2711/mod.rs"]
mod imp;

#[sel4_cfg(all(any(ARCH_RISCV64, ARCH_RISCV32), any(PLAT_SPIKE, PLAT_QEMU_RISCV_VIRT)))]
#[path = "riscv_generic/mod.rs"]
mod imp;

#[allow(unused_imports)]
pub(crate) use imp::*;
