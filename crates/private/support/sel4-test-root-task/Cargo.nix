#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-test-root-task";
  dependencies = {
    inherit (localCrates)
      sel4-test-sentinels
    ;
  };
}
