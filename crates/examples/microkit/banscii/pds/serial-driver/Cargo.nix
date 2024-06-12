#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "banscii-serial-driver";
  dependencies = {
    inherit (localCrates)
      sel4-microkit-message
      sel4-microkit-embedded-hal-adapters
      sel4-pl011-driver
    ;
    sel4-microkit = localCrates.sel4-microkit // { default-features = false; };
  };
}
