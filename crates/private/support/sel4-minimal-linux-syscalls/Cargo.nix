#
# Copyright 2025, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, mk, versions, localCrates }:

mk {
  package.name = "sel4-minimal-linux-syscalls";
  dependencies = {
    syscalls = {
      version = versions.syscalls;
      default-features = false;
    };
    inherit (localCrates)
      sel4-panicking-env
    ;
  };
  target = lib.listToAttrs (lib.forEach [
    "aarch64"
    "arm"
    "riscv64"
    "riscv32"
    "x86_64"
  ] (targetArch: {
    name = "cfg(target_arch = \"${targetArch}\")";
    value = {
      dependencies = {
        syscalls = {
          version = versions.syscalls;
          default-features = false;
          features = [ targetArch ];
        };
      };
    };
  }));
}
