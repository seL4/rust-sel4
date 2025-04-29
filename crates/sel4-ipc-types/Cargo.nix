#
# Copyright 2025, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "sel4-ipc-types";
  dependencies = {
    sel4-microkit-base = localCrates.sel4-microkit-base // {
      optional = true;
    };
  };
}
