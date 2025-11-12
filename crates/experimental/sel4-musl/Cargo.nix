#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-musl";
  dependencies = {
    inherit (localCrates)
      sel4-immediate-sync-once-cell
      sel4-linux-syscall-types
    ;
  };
}
