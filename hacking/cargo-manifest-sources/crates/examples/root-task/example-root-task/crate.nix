#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "example-root-task";
  dependencies = {
    inherit (localCrates)
      sel4
      sel4-root-task
    ;
  };
}
