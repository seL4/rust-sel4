#
# Copyright 2025, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sddf-sys";
  dependencies = {
    inherit (localCrates)
      sel4-sys
    ;
  };
  build-dependencies = {
    inherit (versions)
      bindgen
    ;
    inherit (localCrates)
      sel4-build-env
    ;
  };
}
