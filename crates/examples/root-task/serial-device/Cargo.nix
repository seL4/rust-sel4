#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "serial-device";
  dependencies = {
    inherit (versions) tock-registers;
    inherit (localCrates)
      sel4
      sel4-root-task
    ;
  };
}
