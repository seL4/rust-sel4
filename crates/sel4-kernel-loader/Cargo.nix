#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, objectWith }:

mk {
  package.name = "sel4-kernel-loader";
  package.license = "BSD-2-Clause AND GPL-2.0-only";
  dependencies = {
    inherit (versions)
      log
      embedded-hal-nb
      fdt
    ;
    spin = { version = versions.spin; features = [ "lock_api" ]; };
    inherit (localCrates)
      sel4-platform-info
      sel4-logging
      sel4-config
      sel4-kernel-loader-payload-types
      sel4-stack
      sel4-phdrs
      sel4-phdrs-patched
      sel4-no-allocator
      sel4-immutable-cell
      sel4-immediate-sync-once-cell
    ;
  };
  target."cfg(any(target_arch = \"riscv32\", target_arch = \"riscv64\"))".dependencies = {
    inherit (versions) sbi riscv;
  };
  target."cfg(any(target_arch = \"arm\", target_arch = \"aarch64\"))".dependencies = {
    inherit (versions) smccc;
    inherit (localCrates) sel4-pl011-driver sel4-bcm2835-aux-uart-driver;
  };
  target."cfg(target_arch = \"aarch64\")".dependencies = {
    inherit (versions) aarch64-cpu;
  };
  target."cfg(target_arch = \"arm\")".dependencies = {
    inherit (versions) aarch32-cpu;
  };
  build-dependencies = {
    inherit (versions)
      cc
      glob
    ;
    object = objectWith [];
    inherit (localCrates)
      sel4-config
      sel4-build-env
    ;
  };
  lints.cargo.unused_dependencies = "allow";
}
