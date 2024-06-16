#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "sel4-pl011-driver";
  dependencies = {
    inherit (versions) tock-registers embedded-hal-nb;
    inherit (localCrates) sel4-driver-interfaces;
  };
}
