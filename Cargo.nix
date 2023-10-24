#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, localCrates, defaultFrontmatter }:

{
  nix.frontmatter = defaultFrontmatter;
  workspace = {
    resolver = "2";
    default-members = [];
    members = lib.naturalSort (lib.mapAttrsToList (_: v: v.path) localCrates);
  };
}
