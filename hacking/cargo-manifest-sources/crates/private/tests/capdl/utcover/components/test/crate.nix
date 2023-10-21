#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, serdeWith }:

mk {
  package.name = "tests-capdl-utcover-components-test";
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-sync
    sel4-simple-task-runtime
    sel4-simple-task-config-types
  ];
  dependencies = {
    serde = serdeWith [ "alloc" "derive" ];
  };
}
