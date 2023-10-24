#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "banscii-assistant";
  dependencies = {
    hex = { version = "0.4.3"; default-features = false; features = [ "alloc" ]; };
    inherit (localCrates)
      sel4-microkit-message
      banscii-assistant-core
      banscii-pl011-driver-interface-types
      banscii-artist-interface-types
    ;
    sel4-externally-shared = localCrates.sel4-externally-shared // { features = [ "unstable" ]; };
    sel4-microkit = localCrates.sel4-microkit // { default-features = false; features = [ "alloc" ]; };
  };
}
