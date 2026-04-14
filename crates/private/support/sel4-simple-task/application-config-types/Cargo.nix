#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, serdeWith }:

mk {
  package.name = "sel4-simple-task-application-config-types";
  dependencies = {
    serde = serdeWith [ "derive" ];
    inherit (localCrates)
      sel4
      sel4-simple-task-threading
    ;
  };
}
