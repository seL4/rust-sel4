#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "tests-microkit-unwind";
  dependencies = {
    inherit (localCrates)
      sel4-microkit
    ;
  };
}
