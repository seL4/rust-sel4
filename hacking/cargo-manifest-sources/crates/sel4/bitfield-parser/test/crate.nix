#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-bitfield-parser-test";
  dependencies = {
    inherit (versions) clap;
    glob = "0.3.0";
    inherit (localCrates)
      sel4-bitfield-parser
    ;
  };
}
