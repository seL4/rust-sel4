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
    resolver = "3";
    default-members = [];
    members = lib.naturalSort (lib.mapAttrsToList (_: v: v.path) localCrates);
  };
  patch.crates-io = {
    ring = localCrates.ring or {
      git = "https://github.com/coliasgroup/ring.git";
      rev = "0f749acc5d5a8310dfc3ff985df04056f497fc1b"; # branch sel4
    };
    hashbrown = {
      git = "https://github.com/coliasgroup/hashbrown.git";
      rev = "c284cda40a72b0e9ddb1f9a8ea490d4f73473483"; # branch sel4
    };
  };
}
