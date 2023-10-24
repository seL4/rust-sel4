#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-config-macros";
  lib.proc-macro = true;
  dependencies = {
    inherit (localCrates)
      sel4-config-generic-macro-impls
      sel4-config-data
    ;
  };
}
