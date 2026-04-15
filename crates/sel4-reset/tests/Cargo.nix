#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-reset-tests";
  lib.test = false;
  dependencies = {
    inherit (localCrates)
      sel4-minimal-linux-runtime
      sel4-reset
    ;
  };
  test = [
    {
      name = "test1";
      harness = false;
    }
  ];
}
