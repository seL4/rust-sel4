#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-test-microkit";
  dependencies = {
    inherit (localCrates)
      sel4-microkit
      sel4-test-sentinels
    ;
  };
  features = {
    alloc = [ "sel4-microkit/alloc" ];
  };
}
