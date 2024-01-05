#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, mkDefaultFrontmatterWithReuseArgs, defaultReuseFrontmatterArgs, localCrates, versions, ringWith, rustlsWith }:

mk rec {
  nix.frontmatter = mkDefaultFrontmatterWithReuseArgs (defaultReuseFrontmatterArgs // {
    licenseID = package.license;
  });
  package.name = "sel4-async-network-rustls";
  package.license = "Apache-2.0 OR ISC OR MIT";
  dependencies = {
    inherit (localCrates)
      sel4-async-time
      sel4-async-network
    ;
    inherit (versions) log;
    rustls = rustlsWith [] // (localCrates.rustls or {});
    ring = ringWith [] // (localCrates.ring or {}); # just to force "less-safe-getrandom-custom-or-rdrand" feature
    sel4-newlib = localCrates.sel4-newlib // {
      features = [
        "nosys"
        "all-symbols"
      ];
    };
    getrandom = {
      version = versions.getrandom;
      features = [
        "custom"
      ];
    };
    rand = {
      version = versions.rand;
      default-features = false;
      features = [
        "small_rng"
      ];
    };
    futures = {
      version = versions.futures;
      default-features = false;
      features = [
        "alloc"
      ];
    };
    # TODO remove after bumping rust toolchain
    async-trait = "0.1.73";
  };
}
