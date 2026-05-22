#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "sel4-kernel-loader-masquerade";
  dependencies = {
    inherit (localCrates) sel4-no-panic;
    inherit (versions) fdt;
    semihosting = { version = versions.semihosting; features = [ "stdio" ]; };
  };
}
