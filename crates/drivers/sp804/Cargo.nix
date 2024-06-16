#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "sel4-sp804-driver";
  dependencies = {
    inherit (versions) tock-registers;
    inherit (localCrates) sel4-driver-interfaces;
  };
}
