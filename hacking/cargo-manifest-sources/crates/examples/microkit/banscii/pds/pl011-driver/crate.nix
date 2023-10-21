#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "banscii-pl011-driver";
  dependencies = {
    sel4-microkit.default-features = false;
    inherit (versions) heapless;
  };
  nix.local.dependencies = with localCrates; [
    sel4-microkit
    sel4-microkit-message
    banscii-pl011-driver-core
    banscii-pl011-driver-interface-types
  ];
}
