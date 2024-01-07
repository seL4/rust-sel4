#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "tests-root-task-c";
  dependencies = {
    inherit (localCrates)
      sel4
      sel4-root-task
      sel4-newlib
    ;
  };
  build-dependencies = {
    cc = "1.0.76";
    glob = "0.3.0";
  };
}
