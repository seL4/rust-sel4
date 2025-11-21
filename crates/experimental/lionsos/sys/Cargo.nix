#
# Copyright 2025, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "lionsos-sys";
  package.links = "lions";
  dependencies = {
    inherit (localCrates)
      sddf-sys
    ;
  };
  build-dependencies = {
    inherit (versions)
      bindgen
      cc
    ;
  };
}
