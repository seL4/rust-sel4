#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, volatileSource }:

mk rec {
  package.name = "sel4-externally-shared";
  dependencies = {
    inherit (versions) cfg-if zerocopy;
    volatile = volatileSource;
    inherit (localCrates)
      sel4-atomic-ptr
      # volatile
    ;
  };
  features = {
    "unstable" = [ "volatile/unstable" ];
    "very_unstable" = [ "volatile/very_unstable" ];
  };
}
