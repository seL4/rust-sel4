#
# Copyright 2025, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "sddf-sys";
  package.links = "sddf";
  build-dependencies = {
    inherit (versions)
      bindgen
      cc
    ;
  };
}
