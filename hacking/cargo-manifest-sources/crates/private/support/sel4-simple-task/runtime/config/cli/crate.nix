#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-simple-task-runtime-config-cli";
  dependencies = {
    inherit (versions) serde_json;
    sel4-simple-task-runtime-config-types = localCrates.sel4-simple-task-runtime-config-types // {
      features = [ "serde" "alloc" ];
    };
  };
}
