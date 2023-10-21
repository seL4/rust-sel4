#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-config";
  nix.local.dependencies = with localCrates; [
    sel4-config-macros
  ];
}
