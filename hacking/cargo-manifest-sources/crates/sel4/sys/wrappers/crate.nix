#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-sys-wrappers";
  dependencies = {
    sel4-sys.features = [ "wrappers" ];
  };
  lib.crate-type = [ "staticlib" ];
  nix.local.dependencies = with localCrates; [
    sel4-sys
  ];
}
