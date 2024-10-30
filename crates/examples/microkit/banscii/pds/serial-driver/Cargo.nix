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
      sel4-microkit
      sel4-microkit-message
      sel4-microkit-driver-adapters
    ;
    sel4-pl011-driver = localCrates.sel4-pl011-driver // { optional = true; };
  };
  features = {
    board-qemu_virt_aarch64 = [ "sel4-pl011-driver" ];
  };
}
