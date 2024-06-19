#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "sel4-bcm2835-aux-uart-driver";
  dependencies = {
    inherit (versions) tock-registers embedded-hal-nb;
    inherit (localCrates) sel4-driver-interfaces;
  };
}
