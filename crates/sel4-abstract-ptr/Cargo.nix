#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: MIT OR Apache-2.0
#

{ mk, mkDefaultFrontmatterWithReuseArgs, defaultReuseFrontmatterArgs, versions }:

mk rec {
  nix.frontmatter = mkDefaultFrontmatterWithReuseArgs (defaultReuseFrontmatterArgs // {
    licenseID = package.license;
  });
  package.name = "sel4-abstract-ptr";
  package.license = "MIT OR Apache-2.0";
  dependencies = {
    ptr_meta = { version = versions.ptr_meta; default-features = false; };
  };
}
