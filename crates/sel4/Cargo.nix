#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, mkDefaultFrontmatterWithReuseArgs, defaultReuseFrontmatterArgs, localCrates, versions }:

mk rec {
  nix.frontmatter = mkDefaultFrontmatterWithReuseArgs (defaultReuseFrontmatterArgs // {
    licenseID = package.license;
  });
  package.name = "sel4";
  package.license = "MIT";
  package.rust-version = "1.76"; # tracks verus
  dependencies = {
    inherit (versions) cfg-if;
    inherit (localCrates)
      sel4-config
      sel4-sys
    ;
  };
  features = {
    default = [ "state" ];
    state = [];
    exposed-state = [];
    extern-state = [];
    thread-local-state = [];
    non-thread-local-state = [];
    tls = [];
    single-threaded = [];
  };
}
