#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, serdeWith }:

mk {
  package.name = "tests-capdl-utcover";
  dependencies = {
    serde = serdeWith [ "alloc" "derive" ];
    inherit (localCrates)
      sel4
      sel4-simple-task-runtime
      sel4-simple-task-application-config-types
      sel4-test-capdl
    ;
  };
}
