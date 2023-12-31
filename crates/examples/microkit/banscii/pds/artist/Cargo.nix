#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "banscii-artist";
  dependencies = {
    rsa = { version = "0.8.1"; default-features = false; features = [ "pem" "sha2" ]; };
    inherit (localCrates)
      sel4-microkit-message
      banscii-artist-interface-types
    ;
    sel4-microkit = localCrates.sel4-microkit // { default-features = false; features = [ "alloc" ]; };
    sel4-externally-shared = localCrates.sel4-externally-shared // { features = [ "unstable" ]; };
  };
  build-dependencies = {
    rsa = "0.8.1";
  };
}
