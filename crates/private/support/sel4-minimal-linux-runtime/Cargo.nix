#
# Copyright 2025, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "sel4-minimal-linux-runtime";
  lib.test = false;
  dependencies = {
    inherit (versions)
      one-shot-mutex
    ;
    inherit (localCrates)
      sel4-immediate-sync-once-cell
      sel4-panicking-env
      sel4-dlmalloc
      sel4-minimal-linux-syscalls
      sel4-minimal-linux-runtime-macros
    ;
    sel4-panicking = localCrates.sel4-panicking // { features = [ "personality" "panic-handler" ]; };
    sel4-runtime-common = localCrates.sel4-runtime-common;
  };
  features = {
    full = [
      "alloc"
    ];
    alloc = [
      "sel4-panicking/alloc"
    ];
  };
  test = [
    {
      name = "hello_world";
      harness = false;
    }
  ];
}
