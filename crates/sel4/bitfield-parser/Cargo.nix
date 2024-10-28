#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions }:

mk {
  package.name = "sel4-bitfield-parser";
  dependencies = rec {
    inherit (versions) regex pest;
    pest_derive = pest;
  };
}
