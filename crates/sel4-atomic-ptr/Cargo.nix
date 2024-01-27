#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: MIT OR Apache-2.0
#

{ mk, mkDefaultFrontmatterWithReuseArgs, defaultReuseFrontmatterArgs, versions }:

mk rec {
  nix.frontmatter = mkDefaultFrontmatterWithReuseArgs (defaultReuseFrontmatterArgs // {
    licenseID = package.license;
  });
  package.name = "sel4-atomic-ptr";
  package.license = "MIT OR Apache-2.0";
  dependencies = {
    inherit (versions) cfg-if;
  };
}
