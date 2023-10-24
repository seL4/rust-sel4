#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "microkit-hello";
  dependencies = {
    inherit (localCrates) sel4-microkit;
  };
}
