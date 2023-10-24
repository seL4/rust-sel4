#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-dlmalloc";
  dependencies = {
    dlmalloc = "0.2.3";
    inherit (localCrates)
      sel4-sync
    ;
  };
}
