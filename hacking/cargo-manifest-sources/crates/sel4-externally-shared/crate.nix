#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, volatileSource }:

mk {
  package.name = "sel4-externally-shared";
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
