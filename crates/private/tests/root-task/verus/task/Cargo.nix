#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "tests-root-task-verus-task";
  dependencies = {
    inherit (localCrates)
      sel4
      sel4-root-task
      tests-root-task-verus-core
    ;
  };
}
