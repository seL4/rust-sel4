#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "tests-root-task-c";
  dependencies = {
    inherit (localCrates)
      sel4
      sel4-root-task
    ;
    sel4-newlib = localCrates.sel4-newlib // {
      features = [
        "detect-libc"
        "link-libc"
      ];
    };
  };
  build-dependencies = {
    inherit (versions) cc glob;
  };
}
