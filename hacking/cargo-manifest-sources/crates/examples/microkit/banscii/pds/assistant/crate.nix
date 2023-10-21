#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "banscii-assistant";
  nix.local.dependencies = with localCrates; [
    sel4-microkit
    sel4-microkit-message
    sel4-externally-shared
    banscii-assistant-core
    banscii-pl011-driver-interface-types
    banscii-artist-interface-types
  ];
  dependencies = {
    sel4-microkit = { default-features = false; features = [ "alloc" ]; };
    hex = { version = "0.4.3"; default-features = false; features = [ "alloc" ]; };
    sel4-externally-shared.features = [ "unstable" ];
  };
}
