#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "sel4-pl031-driver";
  dependencies = {
    inherit (versions) tock-registers rtcc;
  };
}
