#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "tests-microkit-reset";
  dependencies = {
    inherit (localCrates)
      sel4-microkit
      sel4-reset
    ;
  };
}
