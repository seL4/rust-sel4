#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates, objectWith }:

mk {
  package.name = "spawn-task";
  dependencies = {
    object = objectWith [];
    inherit (localCrates)
      sel4
      sel4-root-task
    ;
  };
}
