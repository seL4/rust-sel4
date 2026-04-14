#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "tests-microkit-passive-server-with-deferred-action";
  dependencies = {
    inherit (localCrates)
      sel4-microkit
    ;
    sel4-test-microkit = localCrates.sel4-test-microkit // { features = [ "alloc" ]; };
  };
}
