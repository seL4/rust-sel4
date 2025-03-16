#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, unwindingWith }:

mk {
  package.name = "sel4-runtime-common";
  dependencies = {
    inherit (versions) cfg-if;
    inherit (localCrates) sel4-panicking-env sel4-elf-header sel4-stack;
  };
  target."cfg(panic = \"unwind\")".dependencies = {
    unwinding = unwindingWith [];
  };
  target."cfg(target_thread_local)".dependencies = {
    sel4 = localCrates.sel4 // { default-features = false; };
    sel4-initialize-tls = localCrates.sel4-initialize-tls // { features = [ "on-stack" ]; };
  };
  features = {
    full = [ "start" "abort" ];
    start = [];
    abort = [];
  };
}
