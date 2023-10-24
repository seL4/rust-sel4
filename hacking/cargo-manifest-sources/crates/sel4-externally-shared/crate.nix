#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: MIT OR Apache-2.0
#

{ mk, mkDefaultFrontmatterWithReuseArgs, defaultReuseFrontmatterArgs, localCrates, versions, volatileSource }:

mk rec {
  nix.frontmatter = mkDefaultFrontmatterWithReuseArgs (defaultReuseFrontmatterArgs // {
    licenseID = package.license;
  });
  package.name = "sel4-externally-shared";
  package.license = "MIT OR Apache-2.0";
  dependencies = {
    inherit (versions) zerocopy;
    volatile = volatileSource;
    inherit (localCrates)
      # volatile
    ;
  };
  features = {
    "unstable" = [ "volatile/unstable" ];
    "very_unstable" = [ "volatile/very_unstable" ];
  };
}
