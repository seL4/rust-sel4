#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "example-serial-client";
  dependencies = {
    inherit (localCrates)
      sel4-microkit
      sddf
    ;
  };
}
