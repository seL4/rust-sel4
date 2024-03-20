#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "tests-root-task-loader";
  dependencies = {
    inherit (versions) fdt;
    inherit (localCrates)
      sel4
      sel4-root-task
      sel4-platform-info
    ;
  };
}
