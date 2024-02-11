#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-config";
  dependencies = {
    inherit (localCrates)
      sel4-config-macros
    ;
  };
  build-dependencies = {
    inherit (localCrates)
      sel4-config-data
      sel4-config-generic
      sel4-rustfmt-helper
    ;
  };
}
