#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk }:

mk {
  package.name = "sel4-bitfield-parser";
  dependencies = rec {
    regex = "1.7.0";
    pest = "2.4.1";
    pest_derive = pest;
  };
}
