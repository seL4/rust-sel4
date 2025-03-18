#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, mkDefaultFrontmatterWithReuseArgs, defaultReuseFrontmatterArgs }:

mk rec {
  nix.frontmatter = mkDefaultFrontmatterWithReuseArgs (defaultReuseFrontmatterArgs // {
    licenseID = package.license;
  });
  package.name = "sel4-shared-memory";
  package.license = "MIT OR Apache-2.0";
  dependencies = {
    inherit (versions) cfg-if zerocopy;
    aligned = { version = versions.aligned; optional = true; };
    inherit (localCrates)
      sel4-abstract-ptr
    ;
  };
  features = {
    "atomics" = [ "dep:aligned" ];
  };
}
