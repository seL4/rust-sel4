#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: MIT
#

{ mk, mkDefaultFrontmatterWithReuseArgs, defaultReuseFrontmatterArgs, localCrates }:

mk rec {
  nix.frontmatter = mkDefaultFrontmatterWithReuseArgs (defaultReuseFrontmatterArgs // {
    licenseID = package.license;
  });
  package.name = "sel4-simple-task-threading";
  package.license = "MIT";
  dependencies = {
    inherit (localCrates)
      sel4
      sel4-panicking
    ;
  };
  features = {
    alloc = [];
  };
}
