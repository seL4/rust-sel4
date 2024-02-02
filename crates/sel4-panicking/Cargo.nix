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
  target."cfg(not(target_arch = \"arm\"))".dependencies = {
    unwinding = unwindingWith [ "personality" ] // { optional = true; };
  };
  features = {
    alloc = [];
  };
}
