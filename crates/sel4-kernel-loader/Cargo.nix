#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, postcardWith }:

mk {
  package.name = "sel4-kernel-loader";
  package.license = "BSD-2-Clause AND GPL-2.0-only";
  dependencies = {
    inherit (versions) cfg-if log embedded-hal-nb;
    postcard = postcardWith [];
    heapless = { version = versions.heapless; features = [ "serde" ]; };
    spin = { version = "0.9.4"; features = [ "lock_api" ]; };
    inherit (localCrates)
      sel4-platform-info
      sel4-logging
      sel4-config
      sel4-kernel-loader-embed-page-tables-runtime
      sel4-immutable-cell
      sel4-stack
    ;
    sel4-kernel-loader-payload-types = localCrates.sel4-kernel-loader-payload-types // { features = [ "serde" ]; };
  };
  target."cfg(any(target_arch = \"riscv32\", target_arch = \"riscv64\"))".dependencies = {
    sbi = "0.2.0";
    riscv = "0.10.0";
  };
  target."cfg(any(target_arch = \"arm\", target_arch = \"aarch64\"))".dependencies = {
    inherit (localCrates) sel4-pl011-driver sel4-bcm2835-aux-uart-driver;
  };
  target."cfg(target_arch = \"aarch64\")".dependencies = {
    smccc = "0.1.1";
    aarch64-cpu = "9.4.0";
  };
  build-dependencies = {
    inherit (versions)
      proc-macro2
      quote
      object
      serde
      prettyplease
    ;
    postcard = postcardWith [ "alloc" ];
    cc = "1.0.76";
    glob = "0.3.0";
    syn = { version = versions.syn; features = [ "parsing" ]; };
    inherit (localCrates)
      sel4-platform-info
      sel4-config
      sel4-kernel-loader-config-types
      sel4-build-env
      sel4-kernel-loader-payload-types
      sel4-kernel-loader-embed-page-tables
    ;
  };
}
