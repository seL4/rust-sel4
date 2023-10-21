//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

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

pub(crate) trait Plat {
    fn init() {}

    fn init_per_core() {}

    fn put_char(c: u8);

    fn put_char_without_synchronization(c: u8);

    fn start_secondary_core(core_id: usize, sp: usize);
}
