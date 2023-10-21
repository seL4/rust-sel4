#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-backtrace-cli";
  dependencies = {
    sel4-backtrace-types.features = [ "full" ];
    hex = "0.4.3";
    inherit (versions) object addr2line clap;
  };
  nix.local.dependencies = with localCrates; [
    sel4-backtrace-types
  ];
}
