#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, localCrates, defaultFrontmatter }:

{
  nix.frontmatter = defaultFrontmatter;
  nix.formatPolicyOverrides = [
    {
      table_rules = [
        {
          path_regex = ""; # top-level table
          key_ordering.back = [ "patch" ];
        }
      ];
    }
  ];
  workspace = {
    resolver = "2";
    default-members = [];
    members = lib.naturalSort (lib.mapAttrsToList (_: v: v.path) localCrates);
  };
  patch.crates-io = {
    ring = {
      git = "https://github.com/coliasgroup/ring.git";
      rev = "10a2b3cbe68da77f9f20ebb3776ab4c605f2b40e";
    };
  };
}
