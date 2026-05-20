#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "example-root-task-without-runtime";
  dependencies = {
    sel4 = localCrates.sel4 // { default-features = false; };
  };
}
