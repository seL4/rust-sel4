#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, unwindingWith }:

mk {
  package.name = "sel4-panicking";
  dependencies = {
    inherit (versions) cfg-if;
    inherit (localCrates)
      sel4-panicking-env
      sel4-immediate-sync-once-cell
    ;
  };
  build-dependencies = {
    inherit (versions) rustc_version;
  };
  target."cfg(all(panic = \"unwind\", not(target_arch = \"arm\")))".dependencies = {
    unwinding = unwindingWith [ "personality" ];
  };
  features = {
    alloc = [];
  };
}
