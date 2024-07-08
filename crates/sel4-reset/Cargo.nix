#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "sel4-reset";
  dependencies = {
    inherit (versions) cfg-if;
    inherit (localCrates) sel4-stack;
  };
}
