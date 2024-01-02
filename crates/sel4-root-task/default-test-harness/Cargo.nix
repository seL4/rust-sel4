#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk rec {
  package.name = "sel4-root-task-default-test-harness";
  dependencies = {
    inherit (localCrates)
      sel4
      sel4-root-task
      sel4-test-harness
    ;
  };
}
