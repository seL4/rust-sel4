#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, serdeWith }:

mk {
  package.name = "sel4-simple-task-config-types";
  dependencies = {
    inherit (versions) cfg-if;
    serde = serdeWith [ "derive" ];
  };
  target."cfg(target_env = \"sel4\")".dependencies = {
    inherit (localCrates)
      sel4
      sel4-simple-task-threading
    ;
  };
}
