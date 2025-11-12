#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, mkDefaultFrontmatterWithReuseArgs, defaultReuseFrontmatterArgs }:

mk rec {
  nix.frontmatter = mkDefaultFrontmatterWithReuseArgs (defaultReuseFrontmatterArgs // {
    licenseID = package.license;
  });
  package.name = "sel4-test-harness";
  package.license = "MIT OR Apache-2.0";
  dependencies = {
    inherit (localCrates)
      sel4-panicking-env
      sel4-panicking
      sel4-immediate-sync-once-cell
    ;
  };
}
