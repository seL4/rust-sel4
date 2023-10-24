#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: MIT
#

{ mk, localCrates }:

mk rec {
  package.name = "sel4-sync";
  package.license = "MIT";
  nix.reuseFrontmatterArgs.licenseID = package.license;
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-immediate-sync-once-cell
  ];
}
