#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "banscii-assistant";
  dependencies = {
    inherit (versions) embedded-hal-nb;
    hex = { version = versions.hex; default-features = false; features = [ "alloc" ]; };
    inherit (localCrates)
      sel4-microkit-simple-ipc
      sel4-microkit-driver-adapters
      sel4-shared-memory
      banscii-assistant-core
      banscii-artist-interface-types
    ;
    sel4-microkit = localCrates.sel4-microkit // { features = [ "alloc" ]; };
  };
}
