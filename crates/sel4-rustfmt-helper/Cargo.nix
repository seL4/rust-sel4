#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-rustfmt-helper";
  dependencies = {
    which = "4.3.0";
  };
}
