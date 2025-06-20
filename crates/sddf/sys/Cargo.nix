#
# Copyright 2025, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "sddf-sys";
  package.links = "sddf";
  dependencies = {
    ptr_meta = { version = versions.ptr_meta; default-features = false; };
    inherit (localCrates)
      sel4-sys
    ;
  };
  build-dependencies = {
    inherit (versions)
      bindgen
    ;
  };
}
