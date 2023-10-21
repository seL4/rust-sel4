#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-simple-task-runtime-config-cli";
  dependencies = {
    sel4-simple-task-runtime-config-types = { features = [ "serde" "alloc" ]; };
    inherit (versions) serde_json;
  };
  nix.local.dependencies = with localCrates; [
    sel4-simple-task-runtime-config-types
  ];
}
