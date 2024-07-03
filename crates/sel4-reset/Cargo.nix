#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions }:

mk {
  package.name = "sel4-reset";
  dependencies = {
    inherit (versions) cfg-if;
  };
}
