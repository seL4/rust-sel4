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
    ring = localCrates.ring or  {
      git = "https://github.com/coliasgroup/ring.git";
      rev = "c5880ee6ae56bb684f5bb2499f1c05cef8943745"; # branch sel4
    };
  };
}
