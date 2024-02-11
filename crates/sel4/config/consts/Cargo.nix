#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-config-consts";
  build-dependencies = {
    inherit (versions) quote;
    inherit (localCrates)
      sel4-config-data
      sel4-config-generic-types
      sel4-rustfmt-helper
    ;
  };
}
