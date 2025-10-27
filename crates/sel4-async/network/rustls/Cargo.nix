#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, mkDefaultFrontmatterWithReuseArgs, defaultReuseFrontmatterArgs, localCrates, versions, rustlsWith }:

mk rec {
  nix.frontmatter = mkDefaultFrontmatterWithReuseArgs (defaultReuseFrontmatterArgs // {
    licenseID = package.license;
  });
  package.name = "sel4-async-network-rustls";
  package.license = "Apache-2.0 OR ISC OR MIT";
  dependencies = {
    inherit (localCrates)
      sel4-async-io
    ;
    inherit (versions) log embedded-io-async;
    thiserror = { version = versions.thiserror; default-features = false; };
    rustls = rustlsWith [] // (localCrates.rustls or {});
  };
}
