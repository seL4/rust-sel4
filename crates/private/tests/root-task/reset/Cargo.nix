#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "tests-root-task-reset";
  dependencies = {
    inherit (localCrates)
      sel4
      sel4-root-task
      sel4-reset
      sel4-test-root-task
    ;
  };
}
