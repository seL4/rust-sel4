//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_config::sel4_cfg_if;

sel4_cfg_if! {
    if #[sel4_cfg(all(ARCH_ARM, PLAT_QEMU_ARM_VIRT))] {
        #[path = "qemu_arm_virt/mod.rs"]
        mod imp;
    } else if #[sel4_cfg(all(ARCH_ARM, PLAT_BCM2711))] {
        #[path = "bcm2711/mod.rs"]
        mod imp;
    } else if #[sel4_cfg(all(ARCH_RISCV, any(PLAT_SPIKE, PLAT_QEMU_RISCV_VIRT, PLAT_HIFIVE)))] {
        #[path = "riscv_generic/mod.rs"]
        mod imp;
    } else if #[sel4_cfg(all(any(ARCH_AARCH64, ARCH_AARCH32), PLAT_ODROIDC2))] {
        #[path = "odroidc2/mod.rs"]
        mod imp;
    }
}

// HACK for rustfmt
#[cfg(false)]
mod bcm2711;
#[cfg(false)]
mod qemu_arm_virt;
#[cfg(false)]
mod riscv_generic;
#[cfg(any())]
mod odroidc2;
// TODO: need to add meson?

#[allow(unused_imports)]
pub(crate) use imp::*;

pub(crate) trait Plat {
    fn init() {}

    fn init_per_core() {}

    fn put_char(c: u8);

    fn put_char_without_synchronization(c: u8);

    fn start_secondary_core(core_id: usize, sp: usize);
}
