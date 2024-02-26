#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-microkit-base";
  dependencies = {
    inherit (localCrates)
      sel4-immutable-cell
      sel4
    ;
  };
  features = {
    extern-symbols = [];
  };
}
