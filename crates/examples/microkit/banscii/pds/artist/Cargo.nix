#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "banscii-artist";
  dependencies = {
    rsa = { version = versions.rsa; default-features = false; features = [ "pem" "sha2" ]; };
    inherit (localCrates)
      sel4-microkit-message
      sel4-shared-memory
      banscii-artist-interface-types
    ;
    sel4-microkit = localCrates.sel4-microkit // { features = [ "alloc" ]; };
  };
  build-dependencies = {
    inherit (versions) rsa;
  };
}
