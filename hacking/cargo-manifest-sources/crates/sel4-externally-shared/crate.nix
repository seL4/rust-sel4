#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: MIT OR Apache-2.0
#

{ mk, localCrates, versions, volatileSource }:

mk rec {
  package.name = "sel4-externally-shared";
  package.license = "MIT OR Apache-2.0";
  nix.reuseFrontmatterArgs.licenseID = package.license;
  nix.local.dependencies = with localCrates; [
    # volatile
  ];
  dependencies = {
    inherit (versions) zerocopy;
    volatile = volatileSource;
  };
  features = {
    "unstable" = [ "volatile/unstable" ];
    "very_unstable" = [ "volatile/very_unstable" ];
  };
}
