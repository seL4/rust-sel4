#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "microkit-http-server-example-sp804-driver";
  dependencies = {
    inherit (localCrates)
      sel4-microkit
      sel4-microkit-message
      sel4-microkit-driver-adapters
      sel4-driver-interfaces
      sel4-sp804-driver
    ;
  };
}
