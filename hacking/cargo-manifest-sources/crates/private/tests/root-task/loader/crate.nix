#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "tests-root-task-loader";
  dependencies = {
    fdt = "0.1.4";
    inherit (localCrates)
      sel4
      sel4-root-task
      sel4-platform-info
    ;
  };
}
