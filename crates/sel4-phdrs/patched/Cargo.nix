#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-phdrs-patched";
  dependencies = {
    inherit (localCrates)
      sel4-phdrs
      sel4-rodata-static
    ;
  };
}
